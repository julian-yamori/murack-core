/// Mockをまとめた構造体を定義
///
/// # Arguments
/// - $target: テスト対象の構造体の型名
/// - $m: Mockの型名のリスト
/// - $an: Mockではなく個別に注入する変数の名前
/// - $at: Mockではなく個別に注入する変数の型
///
/// # Example
///
/// SongUsecaseImplのテストの際、
/// DbSongRepositoryとFolderUsecaseのMockを使用する場合。
///
/// ```ignore
/// use paste::paste;
/// use std::rc::Rc;
/// use walk_base_2_domain::mocks;
///
/// mocks!{SongUsecaseImpl, [DbSongRepository, FolderUsecase], [config: Rc<Config>]}
/// ```
/// ↓
/// ```ignore
/// use paste::paste;
/// use std::{cell::RefCell, rc::Rc};
/// use walk_base_2_domain::mocks;
///
/// struct Mocks {
///     db_song_repository: Rc<MockDbSongRepository>,
///     folder_usecase: Rc<MockFolderUsecase>,
///     config: Rc<Config>,
/// }
/// impl Mocks {
///     fn new(config: Rc<Config>) -> Self {
///         Self {
///             db_song_repository: Rc::new(MockDbSongRepository::new()),
///             folder_usecase: Rc::new(MockFolderUsecase::new()),
///             config,
///         }
///     }
///     //Mockを取得してセットアップ
///     #[allow(dead_code)]
///     fn db_song_repository<F>(&mut self, body: F)
///     where F: FnOnce(&mut MockDbSongRepository){
///         body(Rc::get_mut(&mut self.db_song_repository).unwrap())
///     }
///     #[allow(dead_code)]
///     fn folder_usecase<F>(&mut self, body: F)
///     where F: FnOnce(&mut MockFolderUsecase){
///         body(Rc::get_mut(&mut self.folder_usecase).unwrap())
///     }
///     //テスト対象の構造体を作成し、テストを実行
///     fn run_target<F>(&mut self, body: F)
///     where F: FnOnce(SongUsecaseImpl) {
///         paste! {
///             let target = SongUsecaseImpl{
///                 db_song_repository: self.db_song_repository.clone(),
///                 folder_usecase: self.folder_usecase.clone(),
///                 config: self.config.clone(),
///             };
///             body(target);
///             self.checkpoint_all();
///         }
///     }
///     fn checkpoint_all(&self) {
///         Rc::get_mut(&mut self.db_song_repository).unwrap().checkpoint();
///         Rc::get_mut(&mut self.folder_usecase).unwrap().checkpoint();
///     }
/// }
/// ```
#[macro_export]
macro_rules! mocks {
    ($target:ident, [$($m:ident),*]$(, [$($an:ident: $at:ty),*])?) => {
        paste! {
            struct Mocks {
                $(
                    [<$m:snake>]: Rc<[<Mock $m>]>,
                )*
                $($(
                    $an: $at,
                )*)?
            }
        }
        impl Mocks {
            fn new($($($an: $at),*)?) -> Self {
                paste! {
                    Self {
                        $(
                            [<$m:snake>]: Rc::new([<Mock $m>]::new()),
                        )*
                        $($(
                            $an,
                        )*)?
                    }
                }
            }
            //Mockを取得してセットアップ
            paste! {
                $(
                    #[allow(dead_code)]
                    fn [<$m:snake>]<F>(&mut self, body: F)
                    where F: FnOnce(&mut [<Mock $m>]){
                        body(Rc::get_mut(&mut self.[<$m:snake>]).unwrap())
                    }
                )*
            }
            //テスト対象の構造体を作成し、テストを実行
            fn run_target<F>(&mut self, body: F)
            where F: FnOnce($target) {
                paste! {
                    let target = $target{
                        $(
                            [<$m:snake>]: self.[<$m:snake>].clone(),
                        )*
                        $($(
                            $an: self.$an.clone(),
                        )*)?
                    };
                    body(target);
                    self.checkpoint_all();
                }
            }
            fn checkpoint_all(&mut self) {
                paste! {
                    $(
                        Rc::get_mut(&mut self.[<$m:snake>]).unwrap().checkpoint();
                    )*
                }
            }
        }
    };
}
