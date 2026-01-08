pub mod cmap;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod maxp;
pub mod name;
pub mod os2;

use crate::font::otf_dtypes::*;

pub enum ParseContext {
    OS2(uint16),
}

pub trait TableTrait {
    fn parse(data: &[u8], ctx: Option<ParseContext>) -> Self
    where
        Self: Sized;

    fn construct(&mut self, data: &[u8]);
}
