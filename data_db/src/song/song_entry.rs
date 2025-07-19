use chrono::NaiveDate;

use super::SongRow;

/// songテーブルの登録用の行データ
pub struct SongEntry<'a> {
    /// 曲の長さ(ミリ秒)
    pub duration: i32,

    /// 曲ファイルのライブラリ内パス
    pub path: &'a str,

    /// フォルダID
    pub folder_id: Option<i32>,

    /// 曲名
    pub title: &'a str,

    /// アーティスト
    pub artist: &'a str,
    /// アルバム
    pub album: &'a str,
    /// ジャンル
    pub genre: &'a str,
    /// アルバムアーティスト
    pub album_artist: &'a str,
    /// 作曲者
    pub composer: &'a str,

    /// トラック番号
    pub track_number: Option<i32>,
    /// トラック最大数
    pub track_max: Option<i32>,

    /// ディスク番号
    pub disc_number: Option<i32>,
    /// ディスク番号(最大)
    pub disc_max: Option<i32>,

    /// リリース日
    pub release_date: Option<NaiveDate>,

    /// レート
    pub rating: i16,

    /// 原曲
    pub original_song: &'a str,

    /// サジェスト対象フラグ
    pub suggest_target: bool,

    /// メモ
    pub memo: &'a str,

    /// 管理メモ
    pub memo_manage: &'a str,

    /// 歌詞
    pub lyrics: &'a str,

    /// 曲名(並び替え用)
    pub title_order: &'a str,
    /// アーティスト(並び替え用)
    pub artist_order: &'a str,
    /// アルバム(並び替え用)
    pub album_order: &'a str,
    /// アルバムアーティスト(並び替え用)
    pub album_artist_order: &'a str,
    /// 作曲者(並び替え用)
    pub composer_order: &'a str,
    /// ジャンル(並び替え用)
    pub genre_order: &'a str,
}

impl SongEntry<'_> {
    /// SongRowの値と等しいことを確認
    pub fn assert_eq_row(&self, row: &SongRow) {
        assert_eq!(self.duration, row.duration);
        assert_eq!(self.path, &row.path);
        assert_eq!(self.folder_id, row.folder_id);
        assert_eq!(self.title, row.title.as_nonnull_str());
        assert_eq!(self.artist, row.artist.as_nonnull_str());
        assert_eq!(self.album, row.album.as_nonnull_str());
        assert_eq!(self.genre, row.genre.as_nonnull_str());
        assert_eq!(self.album_artist, row.album_artist.as_nonnull_str());
        assert_eq!(self.composer, row.composer.as_nonnull_str());
        assert_eq!(self.track_number, row.track_number);
        assert_eq!(self.track_max, row.track_max);
        assert_eq!(self.disc_number, row.disc_number);
        assert_eq!(self.disc_max, row.disc_max);
        assert_eq!(self.release_date, row.release_date);
        assert_eq!(self.rating, row.rating);
        assert_eq!(self.original_song, row.original_track.as_nonnull_str());
        assert_eq!(self.suggest_target, row.suggest_target);
        assert_eq!(self.memo, row.memo.as_nonnull_str());
        assert_eq!(self.memo_manage, row.memo_manage.as_nonnull_str());
        assert_eq!(self.lyrics, row.lyrics.as_nonnull_str());
        assert_eq!(self.title_order, &row.title_order);
        assert_eq!(self.artist_order, &row.artist_order);
        assert_eq!(self.album_order, &row.album_order);
        assert_eq!(self.album_artist_order, &row.album_artist_order);
        assert_eq!(self.composer_order, &row.composer_order);
        assert_eq!(self.genre_order, &row.genre_order);
    }
}
