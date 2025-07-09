use crate::cui::Cui;
use anyhow::Result;
use std::{cell::RefCell, collections::VecDeque, fmt::Arguments, io::Write, rc::Rc};

/// 出力をメモリに保持するCui実装(テスト用)
pub struct BufferCui {
    pub buffer: Rc<RefCell<BufferCuiData>>,
}

impl BufferCui {
    pub fn new() -> Self {
        Self {
            buffer: Rc::new(RefCell::new(BufferCuiData::new())),
        }
    }
}

impl Cui for BufferCui {
    fn out(&self, args: Arguments) {
        self.buffer.borrow_mut().out.write_fmt(args).unwrap();
    }
    fn err(&self, args: Arguments) {
        self.buffer.borrow_mut().err.write_fmt(args).unwrap();
    }

    fn input_case(&self, cases: &[char], message: &str) -> Result<char> {
        let c = self
            .buffer
            .borrow_mut()
            .input
            .pop_front()
            .unwrap_or_else(|| panic!("入力バッファが空\n({message})"));
        assert!(
            !cases.contains(&c),
            "選択肢以外の入力: {c}\n({message})"
        );

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
