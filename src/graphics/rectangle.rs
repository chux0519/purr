use crate::clamp;
use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{Rgba, RgbaImage};
use rand::rngs::SmallRng;
use rand::Rng;
use rand_distr::StandardNormal;

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub p: Point,
    pub x: u32,
    pub y: u32,
}

impl Default for Rectangle {
    fn default() -> Self {
        Rectangle {
            p: Point { x: 0, y: 0 },
            x: 0,
            y: 0,
        }
    }
}

impl Shape for Rectangle {
    fn random(w: u32, h: u32, rng: &mut SmallRng) -> Self {
        let px = rng.gen_range(0, w as i32);
        let py = rng.gen_range(0, h as i32);
        let x = rng.gen_range(0, 32) + 1;
        let y = rng.gen_range(0, 32) + 1;

        Rectangle {
            p: Point { x: px, y: py },
            x,
            y,
        }
    }
    fn mutate(&mut self, w: u32, h: u32, rng: &mut SmallRng) {
        match rng.gen_range(0, 2) {
            0 => {
                self.p.x = clamp(
                    self.p.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    0,
                    w as i32 - 1,
                );
                self.p.y = clamp(
                    self.p.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    0,
                    h as i32 - 1,
                );
            }
            1 => {
                self.x = clamp(
                    self.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
                    1,
                    w - 1,
                );
                self.y = clamp(
                    self.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
                    1,
                    h - 1,
                );
            }
            _ => unreachable!(),
        }
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let points = vec![
            self.p,
            Point {
                x: self.p.x + self.x as i32,
                y: self.p.y,
            },
            Point {
                x: self.p.x + self.x as i32,
                y: self.p.y + self.y as i32,
            },
            Point {
                x: self.p.x,
                y: self.p.y + self.y as i32,
            },
        ];
        scan_polygon(&points, w, h)
    }

    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        let (w, h) = img.dimensions();
        let lines = self.rasterize(w, h);
        for line in lines {
            line.draw(img, &color);
        }
    }

    fn to_svg(&self, attr: &str) -> String {
        format!(
            "<rect {} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
            attr, self.p.x, self.p.y, self.x, self.y
        )
    }
}

impl PurrShape for Rectangle {}

