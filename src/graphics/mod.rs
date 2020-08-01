mod point;
mod scanline;
mod triangle;

pub use point::*;
pub use scanline::*;
pub use triangle::*;

pub trait Shape {
    fn rasterize(&self) -> Vec<Scanline>;
}
