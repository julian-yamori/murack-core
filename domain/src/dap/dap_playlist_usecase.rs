use std::{collections::HashSet, path::Path};

use anyhow::Result;
use async_recursion::async_recursion;
use async_trait::async_trait;
use mockall::mock;
use sqlx::PgPool;

use super::{DapPlaylistObserver, DapRepository, SongFinder};
use crate::{
    db::DbTransaction,
    path::LibSongPath,
    playlist::{DbPlaylistRepository, Playlist},
};

/// DAPとのプレイリスト同期のUsecase
#[async_trait]
pub trait DapPlaylistUsecase {
    /// プレイリスト同期処理を実行
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    /// - all_flag: 変更されていないデータも含め、全て更新するか
    /// - observer: プレイリスト同期処理のObserver
    async fn run<O>(
        &self,
        db_pool: &PgPool,
        dap_plist_path: &Path,
        all_flag: bool,
        observer: &mut O,
    ) -> Result<()>
    where
        O: DapPlaylistObserver + Send + Sync;
}

/// DapPlaylistUsecaseの本実装
#[derive(new)]
pub struct DapPlaylistUsecaseImpl<DR, PR, SF>
where
    DR: DapRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SF: SongFinder + Sync + Send,
{
    dap_repository: DR,
    db_playlist_repository: PR,
    song_finder: SF,
}

#[async_trait]
impl<DR, PR, SF> DapPlaylistUsecase for DapPlaylistUsecaseImpl<DR, PR, SF>
where
    DR: DapRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SF: SongFinder + Sync + Send,
{
    /// プレイリスト同期処理を実行
    /// # Arguments
    /// - dap_plist_path: DAPのプレイリスト保存パス
    /// - all_flag: 変更されていないデータも含め、全て更新するか
    /// - observer: プレイリスト同期処理のObserver
    async fn run<O>(
        &self,
        db_pool: &PgPool,
        dap_plist_path: &Path,
        all_flag: bool,
        observer: &mut O,
    ) -> Result<()>
    where
        O: DapPlaylistObserver + Send + Sync,
    {
        //現在DAPにあるプレイリストファイルを列挙し、Setに格納
        let mut existing_file_set: HashSet<String> = self
            .dap_repository
            .listup_playlist_files(dap_plist_path)?
            .into_iter()
            .collect();

        let mut tx = DbTransaction::PgTransaction {
            tx: db_pool.begin().await?,
        };

        //全て更新するなら、一旦全プレイリストを変更済みとする
        if all_flag {
            self.db_playlist_repository
                .set_dap_change_flag_all(&mut tx, true)
                .await?;
        }

        observer.on_start_load_playlist();

        //プレイリストを全て取得
        let plist_trees = self
            .db_playlist_repository
            .get_playlist_tree(&mut tx)
            .await?;

        //DAPに保存する数を数える
        let save_count = count_save_plists_recursive(&plist_trees);

        observer.on_start_save_file();

        //再帰的に保存を実行
        self.save_plists_recursive(
            &plist_trees,
            &mut tx,
            dap_plist_path,
            0,
            save_count,
            &mut existing_file_set,
        )
        .await?;

        //DBに存在しなかったプレイリストファイルを削除する
        for name in &existing_file_set {
            self.dap_repository
                .delete_playlist_file(dap_plist_path, name)?;
        }

        //DAP未反映フラグを下ろす
        self.db_playlist_repository
            .set_dap_change_flag_all(&mut tx, false)
            .await?;

        tx.commit().await?;

        Ok(())
    }
}

impl<DR, PR, SF> DapPlaylistUsecaseImpl<DR, PR, SF>
where
    DR: DapRepository + Sync + Send,
    PR: DbPlaylistRepository + Sync + Send,
    SF: SongFinder + Sync + Send,
{
    /// プレイリスト数をDAPに再帰的に保存
    ///
    /// # Arguments
    /// - plist_trees: 保存する全プレイリストツリー
    /// - root_path: プレイリストファイルの保存先ディレクトリのパス
    /// - save_offset:  保存する何番目のプレイリストか
    /// - save_count:  全体の保存数
    /// - existingFileSet:  DAPに既に存在するファイルパスのset
    ///
    /// # Returns
    /// プレイリストをいくつ保存したか
    #[async_recursion]
    async fn save_plists_recursive<'c>(
        &self,
        plist_trees: &[Playlist],
        tx: &mut DbTransaction<'c>,
        root_path: &Path,
        save_offset: u32,
        save_count: u32,
        existing_file_set: &mut HashSet<String>,
    ) -> Result<u32> {
        let mut now_save_offset = save_offset;

        let save_count_digit = get_digit(save_count);

        for plist in plist_trees {
            //DAPに保存するプレイリストなら処理
            if plist.save_dap {
                now_save_offset += 1;

                //プレイリストのファイル名を作成
                let plist_file_name =
                    playlist_to_file_name(plist, now_save_offset, save_count_digit);

                //プレイリスト内の曲パスを取得
                let song_paths = self.song_finder.get_song_path_list(tx, plist).await?;

                //プレイリストの曲データ取得後に、リストに変更があったか確認
                let new_plist_data = self
                    .db_playlist_repository
                    .get_playlist(tx, plist.rowid)
                    .await?
                    .expect("playlist not found");

                if new_plist_data.dap_changed {
                    //変更があった場合、保存処理へ進む。

                    //既存ファイルSetから削除
                    if existing_file_set.remove(&plist_file_name) {
                        //見つかって削除できたなら、DAPからも削除
                        self.dap_repository
                            .delete_playlist_file(root_path, &plist_file_name)?;
                    }

                    self.write_playlist_file(root_path, &plist_file_name, &song_paths)?;
                } else {
                    //変更がないなら、上書きする必要なし

                    //既存ファイルSetから削除
                    if !existing_file_set.remove(&plist_file_name) {
                        //もしSetになければ不慮の何かで消えてるので、保存しなおす
                        self.write_playlist_file(root_path, &plist_file_name, &song_paths)?;
                    }
                }
            }
            //子プレイリストの保存
            now_save_offset += self
                .save_plists_recursive(
                    &plist.children,
                    tx,
                    root_path,
                    now_save_offset,
                    save_count,
                    existing_file_set,
                )
                .await?;
        }

        Ok(now_save_offset - save_offset)
    }

    /// プレイリストに曲パスリストを書き込み
    /// # Arguments
    /// - path: プレイリストファイルの保存先パス
    /// - song_path_list: プレイリストファイルに書き込む、曲ファイルパスの一覧
    fn write_playlist_file(
        &self,
        root_path: &Path,
        plist_file_name: &str,
        song_path_list: &[LibSongPath],
    ) -> Result<()> {
        //プレイリストファイルに書き込むデータを作成

        let mut file_data = String::from("#EXTM3U\n");

        for song_path in song_path_list {
            file_data.push_str("#EXTINF:,\n");
            file_data.push_str(SONG_PATH);
            file_data.push_str(song_path.as_str());
            file_data.push('\n')
        }

        //プレイリストファイルを作成する
        self.dap_repository
            .make_playlist_file(root_path, plist_file_name, &file_data)
    }
}

/// 曲ファイルの配置先(プレイリストに記載するルートパス)
/// # todo
/// 外部から設定できるようにする
const SONG_PATH: &str = "lib/";

/// プレイリストファイルの拡張子(ピリオドなし)
/// # todo
/// これも設定から取得したい
const PLAYLIST_EXT: &str = "m3u";

/// DAPに保存するプレイリスト数を再帰的に数える
///
/// # todo
/// modelに移行できそうな処理
fn count_save_plists_recursive(plists: &[Playlist]) -> u32 {
    let mut count = 0;

    for plist in plists {
        if plist.save_dap {
            count += 1;
        }

        count += count_save_plists_recursive(&plist.children);
    }

    count
}

/// 数値の桁数を数える
fn get_digit(mut num: u32) -> u32 {
    //保存する桁数を数える
    let mut digit = 0;
    while num > 0 {
        num /= 10;
        digit += 1;
    }

    digit
}

/// プレイリスト情報から、プレイリストのファイル名を取得
/// # Arguments
/// - plist: パスを取得するプレイリスト
/// - offset: このプレイリストが、全体で何番目か
/// - digit: 保存するプレイリスト数の桁数
///
/// # todo
/// modelに移行できそう
fn playlist_to_file_name(plist: &Playlist, offset: u32, digit: u32) -> String {
    //番号を付ける
    //TODO 書式つかってもっときれいに実装できそう
    let mut buf = offset.to_string();
    //総数の桁数に応じて0埋め
    for _ in 0..(digit - buf.len() as u32) {
        buf.insert(0, '0');
    }

    //親がいるなら追加
    if !plist.parent_names.is_empty() {
        buf = format!("{}-{}", buf, plist.parent_names.join("-"));
    }

    format!("{}-{}.{}", buf, plist.name, PLAYLIST_EXT)
}

#[derive(Default)]
pub struct MockDapPlaylistUsecase {
    pub inner: MockDapPlaylistUsecaseInner,
}
#[async_trait]
impl DapPlaylistUsecase for MockDapPlaylistUsecase {
    async fn run<O>(
        &self,
        db_pool: &PgPool,
        dap_plist_path: &Path,
        all_flag: bool,
        _observer: &mut O,
    ) -> Result<()>
    where
        O: DapPlaylistObserver + Send + Sync,
    {
        self.inner.run(db_pool, dap_plist_path, all_flag)
    }
}
mock! {
    pub DapPlaylistUsecaseInner {
        pub fn run(
            &self,
            db_pool: &PgPool,
            dap_plist_path: &Path,
            all_flag: bool,
        ) -> Result<()>;
    }
}

#[cfg(test)]
mod tests {
    use super::super::{MockDapRepository, MockSongFinder};
    use super::*;
    use crate::{
        mocks,
        playlist::{MockDbPlaylistRepository, PlaylistType, SortType},
    };
    use paste::paste;
    use std::path::PathBuf;
    use test_case::test_case;

    mocks! {
        DapPlaylistUsecaseImpl,
        [
            DapRepository,
            DbPlaylistRepository,
            SongFinder
        ]
    }

    #[test]
    fn test_write_playlist_file() {
        let song_path_list = vec![
            LibSongPath::new("test/hoge/song1.flac"),
            LibSongPath::new("test/song3.m4a"),
            LibSongPath::new("song4.m4a"),
            LibSongPath::new("test/hoge/song2.mp3"),
        ];
        fn root() -> PathBuf {
            "dap_root/test".into()
        }
        const FILE_NAME: &str = "playlist.m3u";

        let mut mocks = Mocks::new();
        mocks.dap_repository(|m| {
            m.expect_make_playlist_file()
                .times(1)
                .returning(|a_root, a_file_name, a_data| {
                    assert_eq!(a_root, &root());
                    assert_eq!(a_file_name, FILE_NAME);
                    assert_eq!(a_data, "#EXTM3U\n#EXTINF:,\nlib/test/hoge/song1.flac\n#EXTINF:,\nlib/test/song3.m4a\n#EXTINF:,\nlib/song4.m4a\n#EXTINF:,\nlib/test/hoge/song2.mp3\n");

                    Ok(())
                });
        });

        mocks.run_target(|t| {
            t.write_playlist_file(&root(), FILE_NAME, &song_path_list)
                .unwrap();
        })
    }

    #[test_case(1, 1 ; "1")]
    #[test_case(9, 1 ; "9")]
    #[test_case(10, 2 ; "10")]
    #[test_case(99, 2 ; "99")]
    #[test_case(100, 3 ; "100")]
    fn test_get_digit(input: u32, expect: u32) {
        assert_eq!(get_digit(input), expect);
    }

    #[test_case("plist", &[], 3, 2, "03-plist.m3u" ; "root")]
    #[test_case("plist", &["parent"], 3, 1, "3-parent-plist.m3u" ; "one_parent_one_digit")]
    #[test_case("plist", &["parent", "2"], 45, 3, "045-parent-2-plist.m3u" ; "two_parents_three_digit")]
    #[test_case("plist-pl", &["parent"], 5, 3, "005-parent-plist-pl.m3u" ; "hyphen_name")]
    fn test_playlist_to_file_name(
        name: &str,
        parents: &[&str],
        offset: u32,
        digit: u32,
        expect: &str,
    ) {
        let plist = Playlist {
            rowid: 3,
            playlist_type: PlaylistType::Normal,
            parent_id: if parents.is_empty() { None } else { Some(34) },
            in_folder_order: 99,
            filter: None,
            sort_type: SortType::Artist,
            sort_desc: false,
            save_dap: true,
            listuped_flag: true,
            dap_changed: true,
            children: Vec::new(),

            name: name.to_owned(),
            parent_names: parents.iter().map(|s| (*s).to_owned()).collect(),
        };
        assert_eq!(&playlist_to_file_name(&plist, offset, digit), expect);
    }
}
