use anyhow::Result;
use sqlx::PgTransaction;

/// DBトランザクションの抽象化
pub enum DbTransaction<'c> {
    // PostgreSQL のトランザクション
    PgTransaction { tx: PgTransaction<'c> },
    //テスト用の、実際にはDB接続を行わないダミーDB接続
    Dummy,
}

impl<'c> DbTransaction<'c> {
    /// トランザクションをコミット
    pub async fn commit(self) -> Result<()> {
        match self {
            Self::PgTransaction { tx } => Ok(tx.commit().await?),
            Self::Dummy => Ok(()),
        }
    }

    /// sqlx の Transaction を取得
    pub fn get(&mut self) -> &mut PgTransaction<'c> {
        match self {
            Self::PgTransaction { tx } => tx,
            Self::Dummy => panic!("Transaction is dummy"),
        }
    }
}
