/// プレイリスト同期処理のObserver
pub trait DapPlaylistObserver {
    /// プレイリスト情報の読み込み開始時
    fn on_start_load_playlist(&mut self);
    /// ファイルの保存開始時
    fn on_start_save_file(&mut self);
}
