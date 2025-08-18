mod command_playlist_model;
mod dap_playlist_repository;
mod file_name;

use std::{collections::HashSet, path::Path};

use anyhow::Result;
use async_recursion::async_recursion;
use murack_core_domain::{
    path::LibraryTrackPath,
    playlist::{PlaylistTree, playlist_sqls},
    track_query::{SelectColumn, playlist_query},
};
use sqlx::{PgPool, PgTransaction};

use crate::{
    Config,
    command::playlist::{command_playlist_model::CommandPlaylistModel, file_name::FileNameContext},
    cui::Cui,
};

/// playlistコマンド
///
/// DAPのプレイリストを更新する
pub struct CommandPlaylist<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    pub config: &'config Config,
    pub cui: &'cui CUI,
}

impl<'config, 'cui, CUI> CommandPlaylist<'config, 'cui, CUI>
where
    CUI: Cui + Send + Sync,
{
    /// このコマンドを実行
    pub async fn run(self, db_pool: &PgPool) -> Result<()> {
        let dap_plist_path = &self.config.dap_playlist;
        let all_flag = false;

        //現在DAPにあるプレイリストファイルを列挙し、Setに格納
        let mut existing_file_set: HashSet<String> =
            dap_playlist_repository::listup_playlist_files(dap_plist_path)?
                .into_iter()
                .collect();

        let mut tx = db_pool.begin().await?;

        //全て更新するなら、一旦全プレイリストを変更済みとする
        if all_flag {
            playlist_sqls::set_dap_change_flag_all(&mut tx, true).await?;
        }

        cui_outln!(self.cui, "プレイリスト情報の取得中...").unwrap();

        // プレイリストを全て取得
        let all_playlists = CommandPlaylistModel::get_all_from_db(&mut tx).await?;

        let plist_trees = PlaylistTree::from_all_playlists(all_playlists)?;

        //DAPに保存する数を数える
        let save_count = count_save_plists_recursive(&plist_trees);

        cui_outln!(self.cui, "プレイリストファイルの保存中...").unwrap();

        //再帰的に保存を実行
        save_plists_recursive(
            &plist_trees,
            &mut tx,
            dap_plist_path,
            &mut FileNameContext::new(save_count),
            &mut existing_file_set,
        )
        .await?;

        //DBに存在しなかったプレイリストファイルを削除する
        for name in &existing_file_set {
            dap_playlist_repository::delete_playlist_file(dap_plist_path, name)?;
        }

        //DAP未反映フラグを下ろす
        playlist_sqls::set_dap_change_flag_all(&mut tx, false).await?;

        tx.commit().await?;

        Ok(())
    }
}

/// プレイリスト数をDAPに再帰的に保存
///
/// # Arguments
/// - plist_trees: 保存する全プレイリストツリー
/// - root_path: プレイリストファイルの保存先ディレクトリのパス
/// - existingFileSet:  DAPに既に存在するファイルパスのset
#[async_recursion]
async fn save_plists_recursive<'c, 'p>(
    plist_trees: &'p [PlaylistTree<CommandPlaylistModel>],
    tx: &mut PgTransaction<'c>,
    root_path: &Path,
    context: &mut FileNameContext<'p>,
    existing_file_set: &mut HashSet<String>,
) -> Result<()> {
    for tree in plist_trees {
        //DAPに保存するプレイリストなら処理
        if tree.value.save_dap {
            //プレイリストのファイル名を作成
            let plist_file_name = file_name::build_file_name(&tree.value.name, context);
            context.offset_of_whole += 1;

            //プレイリスト内の曲パスを取得
            let track_paths: Vec<LibraryTrackPath> =
                playlist_query::select_tracks(tx, tree.value.id, [SelectColumn::Path].into_iter())
                    .await?
                    .into_iter()
                    .map(|row| SelectColumn::row_path(&row))
                    .collect::<Result<Vec<_>, _>>()?;

            //プレイリストの曲データ取得後に、リストに変更があったか確認
            let new_dap_changed = sqlx::query_scalar!(
                "SELECT dap_changed FROM playlists WHERE id = $1",
                tree.value.id
            )
            .fetch_one(&mut **tx)
            .await?;

            if new_dap_changed {
                //変更があった場合、保存処理へ進む。

                //既存ファイルSetから削除
                if existing_file_set.remove(&plist_file_name) {
                    //見つかって削除できたなら、DAPからも削除
                    dap_playlist_repository::delete_playlist_file(root_path, &plist_file_name)?;
                }

                write_playlist_file(root_path, &plist_file_name, &track_paths)?;
            } else {
                //変更がないなら、上書きする必要なし

                //既存ファイルSetから削除
                if !existing_file_set.remove(&plist_file_name) {
                    //もしSetになければ不慮の何かで消えてるので、保存しなおす
                    write_playlist_file(root_path, &plist_file_name, &track_paths)?;
                }
            }
        }

        context.parent_names.push(&tree.value.name);

        //子プレイリストの保存
        save_plists_recursive(&tree.children, tx, root_path, context, existing_file_set).await?;

        context.parent_names.pop();
    }

    Ok(())
}

/// 曲ファイルの配置先(プレイリストに記載するルートパス)
/// # todo
/// 外部から設定できるようにする
const TRACK_PATH: &str = "lib/";

/// プレイリストファイルの拡張子(ピリオドなし)
/// # todo
/// これも設定から取得したい
const PLAYLIST_EXT: &str = "m3u";

/// DAPに保存するプレイリスト数を再帰的に数える
fn count_save_plists_recursive(trees: &[PlaylistTree<CommandPlaylistModel>]) -> u32 {
    let mut count = 0;

    for tree in trees {
        if tree.value.save_dap {
            count += 1;
        }

        count += count_save_plists_recursive(&tree.children);
    }

    count
}

/// プレイリストに曲パスリストを書き込み
/// # Arguments
/// - path: プレイリストファイルの保存先パス
/// - track_path_list: プレイリストファイルに書き込む、曲ファイルパスの一覧
fn write_playlist_file(
    root_path: &Path,
    plist_file_name: &str,
    track_path_list: &[LibraryTrackPath],
) -> Result<()> {
    //プレイリストファイルに書き込むデータを作成

    let mut file_data = String::from("#EXTM3U\n");

    for track_path in track_path_list {
        file_data.push_str("#EXTINF:,\n");
        file_data.push_str(TRACK_PATH);
        file_data.push_str(track_path.as_ref());
        file_data.push('\n')
    }

    //プレイリストファイルを作成する
    dap_playlist_repository::make_playlist_file(root_path, plist_file_name, &file_data)
}

#[cfg(test)]
mod tests {
    use std::{fs, str::FromStr};

    use super::*;

    #[test]
    fn test_write_playlist_file() -> anyhow::Result<()> {
        let track_path_list = vec![
            LibraryTrackPath::from_str("test/hoge/track1.flac")?,
            LibraryTrackPath::from_str("test/track3.m4a")?,
            LibraryTrackPath::from_str("track4.m4a")?,
            LibraryTrackPath::from_str("test/hoge/track2.mp3")?,
        ];
        const FILE_NAME: &str = "playlist.m3u";

        let temp_dir = tempfile::tempdir()?;

        write_playlist_file(temp_dir.path(), FILE_NAME, &track_path_list)?;

        // プレイリストファイルの内容が期待通りか確認
        let playlist_file_path = temp_dir.path().join(FILE_NAME);
        assert_eq!(
            fs::read_to_string(playlist_file_path)?,
            "#EXTM3U\n#EXTINF:,\nlib/test/hoge/track1.flac\n#EXTINF:,\nlib/test/track3.m4a\n#EXTINF:,\nlib/track4.m4a\n#EXTINF:,\nlib/test/hoge/track2.mp3\n"
        );

        Ok(())
    }
}
