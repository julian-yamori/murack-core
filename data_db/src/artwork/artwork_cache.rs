use media::picture::Picture;
use std::sync::Arc;

/// キャッシュされるアートワークのデータ
pub struct ArtworkCachedData {
    /// アートワークID
    pub artwork_id: i32,

    /// アートワーク画像データ
    pub picture: Arc<Picture>,
}

/// アートワークのキャッシュ管理
///
/// 同じアートワークの曲が連続で追加される可能性が高いため、
/// まずこのキャッシュと比較することで、DBアクセスを減らす
#[derive(Default)]
pub struct ArtworkCache {
    /// アートワークのキャッシュ領域
    ///
    /// 同じアートワークの曲が連続で追加される可能性が高いため、
    /// まずこのキャッシュと比較することで、DBアクセスを減らす
    pub cache: Option<ArtworkCachedData>,
}

impl ArtworkCache {
    pub fn new() -> Self {
        Self { cache: None }
    }
}
