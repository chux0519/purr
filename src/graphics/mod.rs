mod point;
mod scanline;
mod triangle;

use crate::{Rgba, RgbaImage};
pub use point::*;
pub use scanline::*;
pub use triangle::*;

use rand::rngs::ThreadRng;

pub trait Shape {
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline>;
    fn random(w: u32, h: u32, rng: &mut ThreadRng) -> Self;
    fn valid(&self) -> bool;
    fn mutate(&mut self, w: u32, h: u32, rng: &mut ThreadRng);
    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>);
}
