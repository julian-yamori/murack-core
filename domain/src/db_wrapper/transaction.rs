use anyhow::Result;
use rusqlite::Transaction;

pub enum TransactionWrapper<'c> {
    Real(Transaction<'c>),
    Dummy { commited: &'c mut bool },
}

impl TransactionWrapper<'_> {
    /// トランザクションをコミット
    pub fn commit(self) -> Result<()> {
        match self {
            Self::Real(t) => Ok(t.commit()?),
            Self::Dummy { commited } => {
                *commited = true;
                Ok(())
            }
        }
    }

    /// rusqlite::Transactionを取得
    pub fn get(&self) -> &Transaction {
        match self {
            Self::Real(t) => t,
            Self::Dummy { commited: _ } => panic!("TransactionWrapper::Dummy transaction get"),
        }
    }
}
