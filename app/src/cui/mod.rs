//! CUI入出力の抽象化

mod cui_trait;
pub use cui_trait::Cui;

mod std_cui;
pub use std_cui::StdCui;

mod buf_cui;
pub use buf_cui::{BufferCui, BufferCuiData};

#[macro_use]
mod macros;
