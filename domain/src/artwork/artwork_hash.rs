use std::error::Error;

use sqlx::{
    Database, Postgres,
    encode::IsNull,
    postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef},
};

/// アートワーク画像のハッシュ値
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtworkHash {
    md5: [u8; 16],
}

impl ArtworkHash {
    pub fn from_image(original_size_image: &[u8]) -> Self {
        Self {
            md5: md5::compute(original_size_image).0,
        }
    }
}

impl AsRef<[u8]> for ArtworkHash {
    fn as_ref(&self) -> &[u8] {
        &self.md5
    }
}

impl sqlx::Type<Postgres> for ArtworkHash {
    fn type_info() -> PgTypeInfo {
        <Vec<u8> as sqlx::Type<Postgres>>::type_info()
    }

    fn compatible(ty: &<Postgres as Database>::TypeInfo) -> bool {
        <Vec<u8> as sqlx::Type<Postgres>>::compatible(ty)
    }
}

impl<'r> sqlx::Decode<'r, Postgres> for ArtworkHash {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <Vec<u8> as sqlx::Decode<Postgres>>::decode(value)?;
        if bytes.len() != 16 {
            return Err(format!("Expected 16 bytes for MD5 hash, got {}", bytes.len()).into());
        }
        let mut md5 = [0u8; 16];
        md5.copy_from_slice(&bytes);
        Ok(ArtworkHash { md5 })
    }
}

impl<'q> sqlx::Encode<'q, Postgres> for ArtworkHash {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        <&[u8] as sqlx::Encode<Postgres>>::encode(&self.md5[..], buf)
    }
}
