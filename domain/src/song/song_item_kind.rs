/// 曲データの値項目の種類
///
/// とりあえずcheckで必要なので、
/// "編集可能部分"にあたる項目のみ用意
pub enum SongItemKind {
    /// 曲名
    Title,
    /// アーティスト
    Artist,
    /// アルバム
    Album,
    /// ジャンル
    Genre,
    /// アルバムアーティスト
    AlbumArtist,
    /// 作曲者
    Composer,
    /// トラック番号
    TrackNumber,
    /// トラック最大数
    TrackMax,
    /// ディスク番号
    DiscNumber,
    /// ディスク番号(最大)
    DiscMax,
    /// リリース日
    ReleaseDate,
    /// メモ
    Memo,
    /// 歌詞
    Lyrics,
}
