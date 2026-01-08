pub mod cmap;
pub mod head;
pub mod hhea;
pub mod hmtx;
pub mod maxp;

pub trait TableTrait {
    fn parse(data: &[u8]) -> Self
    where
        Self: Sized;

    fn construct(&mut self, data: &[u8]);
}
