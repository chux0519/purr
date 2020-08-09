use crate::core::PurrShape;
use crate::graphics::{Ellipse, Point, Scanline, Shape};
use crate::{Rgba, RgbaImage};
use rand::rngs::SmallRng;
use rand::Rng;

#[derive(Debug, Clone, Copy)]
pub struct Circle(Ellipse);

impl Default for Circle {
    fn default() -> Self {
        Circle(Ellipse::default())
    }
}

impl Shape for Circle {
    fn random(w: u32, h: u32, rng: &mut SmallRng) -> Self {
        let x = rng.gen_range(0, w as i32);
        let y = rng.gen_range(0, h as i32);
        let r = rng.gen_range(0, 32) + 1;

        Circle(Ellipse {
            o: Point { x, y },
            rx: r,
            ry: r,
        })
    }

    fn mutate(&mut self, w: u32, h: u32, rng: &mut SmallRng) {
        match rng.gen_range(0, 2) {
            0 => {
                self.0.mutate_o(w, h, rng);
            }
            1 => {
                self.0.mutate_rx(w, rng);
                self.0.ry = self.0.rx;
            }
            2 => {
                self.0.mutate_ry(h, rng);
                self.0.rx = self.0.ry;
            }
            _ => unreachable!(),
        }
    }
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        self.0.rasterize(w, h)
    }
    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        self.0.draw(img, color)
    }

    fn to_svg(&self, attr: &str) -> String {
        self.0.to_svg(attr)
    }
}

impl PurrShape for Circle {}

