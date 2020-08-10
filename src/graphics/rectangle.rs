use crate::clamp;
use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};
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

impl Rectangle {
    fn do_mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, r: u32, rng: &mut T) {
        match r {
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
}

impl Shape for Rectangle {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
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
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        let r = rng.gen_range(0, 2);
        self.do_mutate(w, h, r, rng);
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

#[derive(Debug, Clone, Copy)]
pub struct RotatedRectangle {
    pub degree: u32,
    pub rect: Rectangle,
}

impl Default for RotatedRectangle {
    fn default() -> Self {
        RotatedRectangle {
            degree: 0,
            rect: Rectangle::default(),
        }
    }
}

impl Shape for RotatedRectangle {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        RotatedRectangle {
            degree: rng.gen_range(0, 360),
            rect: Rectangle::random(w, h, rng),
        }
    }
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        self.rect.mutate(w, h, rng);
        match rng.gen_range(0, 3) {
            0 => {
                self.rect.do_mutate(w, h, 0, rng);
            }
            1 => {
                self.rect.do_mutate(w, h, 1, rng);
            }
            2 => {
                // mutate degree
                self.degree += (32.0 * rng.sample::<f64, _>(StandardNormal)) as u32;
            }
            _ => unreachable!(),
        }
    }
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let c = Point {
            x: self.rect.p.x + self.rect.x as i32 / 2,
            y: self.rect.p.y + self.rect.y as i32 / 2,
        };
        let mut p0 = self.rect.p;
        let mut p1 = Point {
            x: self.rect.p.x + self.rect.x as i32,
            y: self.rect.p.y,
        };
        let mut p2 = Point {
            x: self.rect.p.x + self.rect.x as i32,
            y: self.rect.p.y + self.rect.y as i32,
        };
        let mut p3 = Point {
            x: self.rect.p.x,
            y: self.rect.p.y + self.rect.y as i32,
        };
        rotate_point(&c, &mut p0, self.degree as f32);
        rotate_point(&c, &mut p1, self.degree as f32);
        rotate_point(&c, &mut p2, self.degree as f32);
        rotate_point(&c, &mut p3, self.degree as f32);
        let points = vec![p0, p1, p2, p3];
        let lines = scan_polygon(&points, w, h);
        let mut visible_lines: Vec<Scanline> = lines
            .into_iter()
            .filter(|l| l.x1 <= l.x2 && l.x2 > 0 && l.x1 < w && l.y > 0 && l.y < h)
            .collect();
        for line in &mut visible_lines {
            line.crop(w, h);
        }
        visible_lines
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
            "<g transform=\"translate({} {}) rotate({} {} {}) scale({} {})\"><rect {} x=\"0\" y=\"0\" width=\"1\" height=\"1\" /></g>",
		    self.rect.p.x, self.rect.p.y, self.degree, self.rect.x / 2, self.rect.y / 2,self.rect.x, self.rect.y, attr
        )
    }
}

impl PurrShape for RotatedRectangle {}

fn rotate_point(c: &Point, p: &mut Point, degree: f32) {
    let cos = (degree * std::f32::consts::PI / 180.0).cos();
    let sin = (degree * std::f32::consts::PI / 180.0).sin();
    let new_x = (cos * (p.x - c.x) as f32 - sin * (p.y - c.y) as f32) as i32 + c.x;
    let new_y = (sin * (p.x - c.x) as f32 + cos * (p.y - c.y) as f32) as i32 + c.y;
    p.x = new_x;
    p.y = new_y;
}
