pub mod cmap;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod loca;
pub mod maxp;
pub mod name;
pub mod os2;
pub mod post;

use crate::font::otf_dtypes::*;

pub enum ParseContext {
    OS2(uint16),
    Loca((int16, uint16)),
}

pub trait TableTrait {
    fn parse(data: &[u8], ctx: Option<ParseContext>) -> Self
    where
        Self: Sized;

    fn construct(&mut self, data: &[u8]);
}
