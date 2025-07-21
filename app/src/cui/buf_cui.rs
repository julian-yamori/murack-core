use std::{
    collections::VecDeque,
    fmt::Arguments,
    io::Write,
    sync::{Arc, Mutex, MutexGuard},
};

use anyhow::{Result, anyhow};

use crate::cui::Cui;

/// 出力をメモリに保持するCui実装(テスト用)
pub struct BufferCui {
    pub buffer: Arc<Mutex<BufferCuiData>>,
}

impl BufferCui {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(BufferCuiData::new())),
        }
    }

    fn lock_buffer(&self) -> Result<MutexGuard<BufferCuiData>> {
        self.buffer
            .lock()
            .map_err(|_| anyhow!("buffer cache lock error"))
    }
}

impl Cui for BufferCui {
    fn out(&self, args: Arguments) {
        self.lock_buffer().unwrap().out.write_fmt(args).unwrap();
    }
    fn err(&self, args: Arguments) {
        self.lock_buffer().unwrap().err.write_fmt(args).unwrap();
    }

    fn input_case(&self, cases: &[char], message: &str) -> Result<char> {
        let c = self
            .lock_buffer()
            .unwrap()
            .input
            .pop_front()
            .unwrap_or_else(|| panic!("入力バッファが空\n({message})"));
        assert!(!cases.contains(&c), "選択肢以外の入力: {c}\n({message})");

        Ok(c)
    }
}

impl Default for BufferCui {
    fn default() -> Self {
        Self::new()
    }
}

/// CUIへの入出力をメモリ上に保存する構造体
pub struct BufferCuiData {
    /// 標準出力
    pub out: Vec<u8>,
    /// 標準エラー
    pub err: Vec<u8>,
    /// 文字入力選択の予約入力
    pub input: VecDeque<char>,
}

impl BufferCuiData {
    pub fn new() -> Self {
        Self {
            out: vec![],
            err: vec![],
            input: VecDeque::new(),
        }
    }
}

impl Default for BufferCuiData {
    fn default() -> Self {
        Self::new()
    }
}
