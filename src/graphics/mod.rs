mod point;
mod scanline;
mod triangle;

pub use point::*;
pub use scanline::*;
pub use triangle::*;

use rand::rngs::ThreadRng;

pub trait Shape {
    fn rasterize(&self) -> Vec<Scanline>;
    fn random(w: u32, h: u32, rng: &mut ThreadRng) -> Self;
    fn valid(&self) -> bool;
    fn mutate(&mut self, w: u32, h: u32, rng: &mut ThreadRng);
}
