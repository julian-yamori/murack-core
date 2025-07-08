use super::TransactionWrapper;
use anyhow::Result;
use rusqlite::Connection;

pub enum ConnectionWrapper {
    Real(Connection),
    Dummy { commited: bool },
}

impl ConnectionWrapper {
    /// transactionを開始
    pub fn transaction(&mut self) -> Result<TransactionWrapper> {
        match self {
            Self::Real(c) => Ok(TransactionWrapper::Real(c.transaction()?)),
            Self::Dummy { commited: c } => Ok(TransactionWrapper::Dummy { commited: c }),
        }
    }

    /// トランザクション内で処理を実行
    ///
    /// トランザクションを開始し、bodyの処理を実行する。
    /// bodyが成功したらコミットし、bodyの戻り値を返す。
    pub fn run_in_transaction<'c, R, F>(&'c mut self, body: F) -> Result<R>
    where
        F: FnOnce(&TransactionWrapper<'c>) -> Result<R>,
    {
        let tx = self.transaction()?;
        let ret_val = body(&tx)?;
        tx.commit()?;
        Ok(ret_val)
    }

    /// rusqlite::Connectionを取得
    pub fn get(&mut self) -> &mut Connection {
        match self {
            Self::Real(c) => c,
            Self::Dummy { commited: _ } => panic!("ConnectionWrapper::Dummy get"),
        }
    }

    /// コミットされたかどうかを取得(テスト用)
    pub fn is_commited(&self) -> bool {
        match self {
            Self::Real(_) => panic!("ConnectionWrapper::Real is_commited"),
            Self::Dummy { commited } => *commited,
        }
    }
}
