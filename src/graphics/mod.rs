mod circle;
mod combo;
mod ellipse;
mod point;
mod polygon;
mod quadratic;
mod raster;
mod rectangle;
mod scanline;
mod triangle;

use crate::{Rgba, RgbaImage};
pub use circle::*;
pub use combo::*;
pub use ellipse::*;
pub use point::*;
pub use polygon::*;
pub use quadratic::*;
pub use raster::*;
pub use rectangle::*;
pub use scanline::*;
pub use triangle::*;

use rand::{RngCore, SeedableRng};

pub trait Shape {
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline>;
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self;
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T);
    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>);
    fn to_svg(&self, attr: &str) -> String;
}
