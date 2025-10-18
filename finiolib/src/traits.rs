//! Унифицированные трэйты чтения/записи на основе std::io::{BufRead, Write}.

use crate::{error::Result, model::Statement};
use std::io::{BufRead, Write};

pub trait ReadFormat {
    fn read<R: BufRead>(r: R) -> Result<Statement>;
}

pub trait WriteFormat {
    fn write<W: Write>(w: W, st: &Statement) -> Result<()>;
}

pub trait Format: ReadFormat + WriteFormat {}
impl<T: ReadFormat + WriteFormat> Format for T {}
