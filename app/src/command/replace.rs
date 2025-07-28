use crate::{AppComponents, Config, Error, cui::Cui};
use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    rc::Rc,
};
use walk_base_2_domain::{
    FileLibraryRepository,
    db_wrapper::{ConnectionFactory, ConnectionWrapper},
    folder::{DbFolderRepository, FolderUsecase},
    path::LibraryTrackPath,
    playlist::DbPlaylistRepository,
    sync::DbTrackSyncRepository,
    track::DbTrackRepository,
};

/// replaceコマンド
///
/// ライブラリ内の曲ファイルを、別のファイルで置き換える
pub struct CommandReplace {
    args: Args,

    config: Rc<Config>,
    cui: Rc<dyn Cui>,
    connection_factory: Rc<ConnectionFactory>,
    file_library_repository: Rc<dyn FileLibraryRepository>,
    db_folder_repository: Rc<dyn DbFolderRepository>,
    db_playlist_repository: Rc<dyn DbPlaylistRepository>,
    db_track_repository: Rc<dyn DbTrackRepository>,
    db_track_sync_repository: Rc<dyn DbTrackSyncRepository>,
    folder_usecase: Rc<dyn FolderUsecase>,
}

impl CommandReplace {
    pub fn new(args: Args, app_components: &impl AppComponents) -> Self {
        Self {
            args,
            config: app_components.config().clone(),
            cui: app_components.cui().clone(),
            connection_factory: app_components.connection_factory().clone(),
            file_library_repository: app_components.file_library_repository().clone(),
            db_folder_repository: app_components.db_folder_repository().clone(),
            db_playlist_repository: app_components.db_playlist_repository().clone(),
            db_track_repository: app_components.db_track_repository().clone(),
            db_track_sync_repository: app_components.db_track_sync_repository().clone(),
            folder_usecase: app_components.folder_usecase().clone(),
        }
    }

    /// このコマンドを実行
    pub fn run(&self) -> Result<()> {
        let mut db = self.connection_factory.open()?;

        //作業対象のファイルを列挙
        let listup_result = self.listup_files(&mut db)?;

        //取得した全ファイルについて処理
        let unit_list_len = listup_result.unit_list.len();
        for (idx, unit) in listup_result.unit_list.iter().enumerate() {
            self.write_console_progress(idx + 1, unit_list_len, unit);

            if let Err(e) = self.unit_replace(&mut db, unit) {
                //失敗したらコンソールに出力して次の曲へ
                self.cui.err(format_args!("{}\n", e));
            }
        }

        cui_outln!(self.cui)?;

        //ライブラリ外側の見つからなかった曲があれば列挙
        if !listup_result.lib_not_founds.is_empty() {
            for path in listup_result.lib_not_founds {
                self.cui.err(format_args!(
                    "File not exists in library: \"{}\"\n",
                    path.display()
                ))
            }
            cui_outln!(self.cui)?;
        }

        //差し替えられなかったライブラリ内の曲があれば列挙
        if !listup_result.remain_lib_tracks.is_empty() {
            for path in listup_result.remain_lib_tracks {
                self.cui
                    .err(format_args!("{} dosn't exist in source.\n", path))
            }
            cui_outln!(self.cui)?;
        }

        Ok(())
    }

    /// 作業対象のファイルを列挙
    fn listup_files(&self, db: &mut ConnectionWrapper) -> Result<ListupResult> {
        let mut result = ListupResult::default();
        //指定されたパスから、新規ファイルのパスを列挙
        let src_file_list = self
            .file_library_repository
            .search_track_outside_lib(&self.args.new_file_path)?;

        //引数で指定された差し替え先の曲パスリストをDBから取得
        result.remain_lib_tracks = db.run_in_transaction(|tx| {
            self.db_track_repository
                .get_path_by_path_str(tx, &self.args.dest_path)
        })?;

        //todo ファイル直接指定の場合は、ファイル名一致する必要ない仕様にしたい

        //新規ファイルごとに繰り返し
        result.unit_list.reserve(src_file_list.len());
        for new_file_path in src_file_list {
            //DBから見つかった曲パスリストから検索
            match self.find_lib_track_from_name(&mut result.remain_lib_tracks, &new_file_path)? {
                Some(old_lib_path) => {
                    result.unit_list.push(OpeUnit {
                        new_lib_path: track_ext_from_others(&old_lib_path, &new_file_path)?,
                        new_file_path,
                        old_lib_path,
                    });
                }
                //見つからなければ、見つからなかったリストへ
                None => {
                    result.lib_not_founds.push(new_file_path);
                }
            }
        }

        Ok(result)
    }

    /// ライブラリの曲リストから、ファイル名が一致する曲データを探す
    ///
    /// 見つかった場合はlibTrackListから削除する。
    ///
    /// # Arguments
    /// - libTrackList: 検索元の曲データリスト
    /// - srcFileName: 差し替える新規ファイルのパス
    ///
    /// # Returns
    /// 該当する曲の情報(見つからなければNone)
    fn find_lib_track_from_name(
        &self,
        lib_track_list: &mut Vec<LibraryTrackPath>,
        src_file_path: &Path,
    ) -> Result<Option<LibraryTrackPath>> {
        //todo ファイル名だけで判断してるので、引数より下のディレクトリ構造が無視される。
        // 一旦このままでいいかな…

        //新規ファイル側の拡張子なしファイル名
        let src_no_ext = src_file_path.file_stem().with_context(|| {
            format!(
                "拡張子なしのファイル名取得に失敗: {}",
                src_file_path.display()
            )
        })?;

        //リスト内から、拡張子なしファイル名が一致するインデックスを検索
        match lib_track_list
            .iter()
            .position(|path| path.file_stem() == src_no_ext)
        {
            //リストから削除して取得
            Some(find_idx) => Ok(Some(lib_track_list.remove(find_idx))),
            //見つからなければNoneを返す
            None => Ok(None),
        }
    }

    /// 1曲の差し替え処理
    fn unit_replace(&self, db: &mut ConnectionWrapper, unit: &OpeUnit) -> Result<()> {
        //PCのファイルを差し替え
        self.replace_pc_file(unit)?;

        //PCのファイルのメタデータをDBから反映
        self.apply_metadata(db, unit)?;

        //DAPのファイルを差し替え
        self.replace_dap_file(unit)?;

        Ok(())
    }

    /// PCのライブラリの曲ファイルを差し替え
    /// # Arguments
    /// - unit: 作業対象の曲ファイルの情報
    fn replace_pc_file(&self, unit: &OpeUnit) -> Result<()> {
        //既存の曲ファイルをゴミ箱に移動
        self.file_library_repository
            .trash_track(&self.config.pc_lib, &unit.old_lib_path)?;

        self.file_library_repository.copy_from_outside_lib(
            &self.config.pc_lib,
            &unit.new_lib_path,
            &unit.new_file_path,
        )?;

        Ok(())
    }

    /// DBのメタデータを新規ファイルに反映
    fn apply_metadata(&self, db: &mut ConnectionWrapper, unit: &OpeUnit) -> Result<()> {
        //ファイルからメタデータを読み込み
        let file_track = self
            .file_library_repository
            .read_track_sync(&self.config.pc_lib, &unit.new_lib_path)?;

        let db_track = db.run_in_transaction(|tx| {
            //新規パスの親ディレクトリを登録してIDを取得
            let new_folder_id = if let Some(new_parent) = unit.new_lib_path.parent() {
                self.db_folder_repository
                    .register_not_exists(tx, &new_parent)?;
            } else {
                FolderIdMayRoot::Root
            };

            //DBにパスの変更を反映
            //todo usecase層でフォルダ登録の処理もした方が良さそう。
            self.db_track_repository.update_path(
                tx,
                &unit.old_lib_path,
                &unit.new_lib_path,
                new_folder_id,
            )?;

            //旧パスの親ディレクトリが無くなるなら削除
            if let Some(parent) = unit.old_lib_path.parent() {
                self.folder_usecase.delete_db_if_empty(tx, &parent)?;
            }

            let db_track = self
                .db_track_sync_repository
                .get_by_path(tx, &unit.new_lib_path)?
                .ok_or_else(|| {
                    walk_base_2_domain::Error::DbTrackNotFound(unit.new_lib_path.to_owned())
                })?;

            //再生時間を反映
            self.db_track_repository
                .update_duration(tx, db_track.id, file_track.duration)?;

            //プレイリストのリストアップ済みフラグを解除し、DAP変更フラグを立てる
            self.db_playlist_repository.reset_listuped_flag(tx)?;
            self.db_playlist_repository
                .set_dap_change_flag_all(tx, true)?;

            Ok(db_track)
        })?;

        //ファイルにDBの内容を反映
        self.file_library_repository.overwrite_track_sync(
            &self.config.pc_lib,
            &unit.new_lib_path,
            &db_track.track_sync,
        )?;

        Ok(())
    }

    /// DAPのライブラリの曲ファイルを差し替え
    /// # Arguments
    /// - unit: 作業対象の曲ファイルの情報
    fn replace_dap_file(&self, unit: &OpeUnit) -> Result<()> {
        self.file_library_repository
            .delete_track(&self.config.dap_lib, &unit.old_lib_path)?;
        self.file_library_repository.copy_track_over_lib(
            &self.config.pc_lib,
            &self.config.dap_lib,
            &unit.new_lib_path,
        )?;

        Ok(())
    }

    /// コンソールに進捗を出力
    ///
    /// # Arguments
    /// - current_idx: 何曲目の操作中か(1から始まる)
    /// - all_count: 全部で何曲あるか
    /// - unit: 作業対象の情報
    fn write_console_progress(&self, current_idx: usize, all_count: usize, unit: &OpeUnit) {
        let new_file_name = match unit.new_file_path.file_name() {
            Some(osstr) => match osstr.to_str() {
                Some(s) => s.to_owned(),
                None => "(failed to convert UTF-8)".to_owned(),
            },
            None => "(no file name)".to_owned(),
        };

        cui_outln!(
            self.cui,
            "({}/{}) {} => {}",
            current_idx,
            all_count,
            new_file_name,
            unit.old_lib_path
        )?;
    }
}

/// LibraryTrackPathの拡張子を他のパスのものに変更
fn track_ext_from_others(track_path: &LibraryTrackPath, others: &Path) -> Result<LibraryTrackPath> {
    match others.extension() {
        Some(ext) => {
            let ext_utf8 = ext
                .to_str()
                .with_context(|| format!("拡張子のUTF-8への変換に失敗: {}", others.display()))?;
            Ok(track_path.with_extension(ext_utf8))
        }
        //差し替え元に拡張子がなければ、とりあえず変更なし
        None => Ok(track_path.clone()),
    }
}

/// 引数の曲パスの検索結果
#[derive(Default, Debug, PartialEq)]
struct ListupResult {
    /// 作業対象のファイル情報のリスト
    unit_list: Vec<OpeUnit>,
    /// 上書きされないライブラリ内の曲リスト
    remain_lib_tracks: Vec<LibraryTrackPath>,
    /// DBに見つからなかった、ライブラリ外のファイルパス
    lib_not_founds: Vec<PathBuf>,
}

/// 1曲の処理についての情報
#[derive(Debug, PartialEq)]
struct OpeUnit {
    /// 差し替える新規ファイルのパス
    new_file_path: PathBuf,

    /// 差し替え前のライブラリ内パス
    ///
    /// 該当するデータが見つからない場合はNone
    old_lib_path: LibraryTrackPath,

    /// 差し替え後の拡張子を変更したライブラリ内パス
    ///
    /// 該当するデータが見つからない場合はNone
    /// # todo
    /// Optionにする必要ないかも
    new_lib_path: LibraryTrackPath,
}

/// コマンドの引数
#[derive(Debug, PartialEq, Clone)]
pub struct Args {
    /// 差し替え先のパス
    ///
    /// ディレクトリ指定可(new_file_pathもディレクトリである必要あり)
    pub dest_path: NonEmptyString,

    /// 新規ファイルのパス
    ///
    /// ディレクトリ指定可(destPathもディレクトリである必要あり)
    pub new_file_path: PathBuf,
}

impl Args {
    /// コマンドの引数を解析
    pub fn parse(command_line: &[String]) -> Result<Args> {
        match command_line {
            [src, dest, ..] => Ok(Args {
                new_file_path: src.into(),
                dest_path: dest.clone().into(),
            }),
            [_] => Err(Error::InvalidCommandArgument {
                msg: "destination path is not specified.".to_owned(),
            }
            .into()),
            [] => Err(Error::InvalidCommandArgument {
                msg: "source file path is not specified.".to_owned(),
            }
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cui::BufferCui;
    use paste::paste;
    use walk_base_2_domain::{
        MockFileLibraryRepository,
        folder::{MockDbFolderRepository, MockFolderUsecase},
        mocks,
        playlist::MockDbPlaylistRepository,
        sync::MockDbTrackSyncRepository,
        track::MockDbTrackRepository,
    };

    mocks! {
        CommandReplace,
        [FileLibraryRepository, DbFolderRepository, DbPlaylistRepository, DbTrackRepository, DbTrackSyncRepository, FolderUsecase],
        [args: Args, config: Rc<Config>, cui: Rc<BufferCui>, connection_factory: Rc<ConnectionFactory>]
    }

    fn new_mocks(new_file_path: &str, dest_path: &str) -> Mocks {
        Mocks::new(
            Args {
                new_file_path: PathBuf::from(new_file_path),
                dest_path: NonEmptyString::from(dest_path.to_owned()),
            },
            Rc::new(Config::dummy()),
            Rc::new(BufferCui::new()),
            Rc::new(ConnectionFactory::Dummy),
        )
    }

    /// listup_files: ファイル直接
    #[test]
    fn test_listup_files_track() {
        let mut mocks = new_mocks("/home/taro/test.flac", "folder/test.mp3");

        mocks.file_library_repository(|m| {
            m.expect_search_track_outside_lib()
                .returning(|_| Ok(vec![PathBuf::from("/home/taro/test.flac")]));
        });
        mocks.db_track_repository(|m| {
            m.expect_get_path_by_path_str()
                .returning(|_, _| Ok(vec![LibraryTrackPath::new("folder/test.mp3")]));
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(
                target.listup_files(&mut db).unwrap(),
                ListupResult {
                    unit_list: vec![OpeUnit {
                        new_file_path: PathBuf::from("/home/taro/test.flac"),
                        old_lib_path: LibraryTrackPath::new("folder/test.mp3"),
                        new_lib_path: LibraryTrackPath::new("folder/test.flac"),
                    }],
                    lib_not_founds: vec![],
                    remain_lib_tracks: vec![],
                }
            )
        });
    }

    /// listup_files: ルート直下のファイル指定
    #[test]
    fn test_listup_files_root() {
        let mut mocks = new_mocks("/home/taro/test.flac", "test.mp3");

        mocks.file_library_repository(|m| {
            m.expect_search_track_outside_lib()
                .returning(|_| Ok(vec![PathBuf::from("/home/taro/test.flac")]));
        });
        mocks.db_track_repository(|m| {
            m.expect_get_path_by_path_str()
                .returning(|_, _| Ok(vec![LibraryTrackPath::new("test.mp3")]));
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(
                target.listup_files(&mut db).unwrap(),
                ListupResult {
                    unit_list: vec![OpeUnit {
                        new_file_path: PathBuf::from("/home/taro/test.flac"),
                        old_lib_path: LibraryTrackPath::new("test.mp3"),
                        new_lib_path: LibraryTrackPath::new("test.flac"),
                    }],
                    lib_not_founds: vec![],
                    remain_lib_tracks: vec![],
                }
            )
        });
    }

    /// listup_files: ディレクトリ
    #[test]
    fn test_listup_files_dir() {
        let mut mocks = new_mocks("/home/taro/musics", "folder/under");

        mocks.file_library_repository(|m| {
            m.expect_search_track_outside_lib().returning(|_| {
                Ok(vec![
                    PathBuf::from("/home/taro/track1.flac"),
                    PathBuf::from("/home/taro/track2.flac"),
                    PathBuf::from("/home/taro/track3.flac"),
                ])
            });
        });
        mocks.db_track_repository(|m| {
            m.expect_get_path_by_path_str().returning(|_, _| {
                Ok(vec![
                    LibraryTrackPath::new("folder/under/track1.mp3"),
                    LibraryTrackPath::new("folder/under/track3.mp3"),
                    LibraryTrackPath::new("folder/under/track2.flac"),
                ])
            });
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(
                target.listup_files(&mut db).unwrap(),
                ListupResult {
                    unit_list: vec![
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track1.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track1.mp3"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track1.flac"),
                        },
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track2.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track2.flac"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track2.flac"),
                        },
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track3.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track3.mp3"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track3.flac"),
                        },
                    ],
                    lib_not_founds: vec![],
                    remain_lib_tracks: vec![],
                }
            )
        });
    }

    /// listup_files: 過不足あり
    #[test]
    fn test_listup_files_remain() {
        let mut mocks = new_mocks("/home/taro/musics", "folder/under");

        mocks.file_library_repository(|m| {
            m.expect_search_track_outside_lib().returning(|_| {
                Ok(vec![
                    PathBuf::from("/home/taro/f_rem1.flac"),
                    PathBuf::from("/home/taro/track1.flac"),
                    PathBuf::from("/home/taro/track2.flac"),
                    PathBuf::from("/home/taro/track3.flac"),
                    PathBuf::from("/home/taro/f_rem2.flac"),
                ])
            });
        });
        mocks.db_track_repository(|m| {
            m.expect_get_path_by_path_str().returning(|_, _| {
                Ok(vec![
                    LibraryTrackPath::new("folder/under/d_rem1.mp3"),
                    LibraryTrackPath::new("folder/under/track1.mp3"),
                    LibraryTrackPath::new("folder/under/track3.mp3"),
                    LibraryTrackPath::new("folder/under/track2.flac"),
                    LibraryTrackPath::new("folder/under/d_rem2.mp3"),
                ])
            });
        });

        let mut db = mocks.connection_factory.open().unwrap();

        mocks.run_target(|target| {
            assert_eq!(
                target.listup_files(&mut db).unwrap(),
                ListupResult {
                    unit_list: vec![
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track1.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track1.mp3"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track1.flac"),
                        },
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track2.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track2.flac"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track2.flac"),
                        },
                        OpeUnit {
                            new_file_path: PathBuf::from("/home/taro/track3.flac"),
                            old_lib_path: LibraryTrackPath::new("folder/under/track3.mp3"),
                            new_lib_path: LibraryTrackPath::new("folder/under/track3.flac"),
                        },
                    ],
                    lib_not_founds: vec![
                        PathBuf::from("/home/taro/rem1.flac"),
                        PathBuf::from("/home/taro/rem2.flac"),
                    ],
                    remain_lib_tracks: vec![
                        LibraryTrackPath::new("folder/under/rem1.mp3"),
                        LibraryTrackPath::new("folder/under/rem2.mp3"),
                    ],
                }
            )
        });
    }
}
