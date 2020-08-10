use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{clamp, degrees};
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};
use rand_distr::StandardNormal;

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: Point,
    pub b: Point,
    pub c: Point,
}

impl Default for Triangle {
    fn default() -> Self {
        Triangle {
            a: Point { x: 0, y: 0 },
            b: Point { x: 0, y: 0 },
            c: Point { x: 0, y: 0 },
        }
    }
}

impl Triangle {
    fn valid(&self) -> bool {
        let min_degree = 15.0;
        let mut x1 = (self.b.x - self.a.x) as f64;
        let mut y1 = (self.b.y - self.a.y) as f64;
        let mut x2 = (self.c.x - self.a.x) as f64;
        let mut y2 = (self.c.y - self.a.y) as f64;
        let mut d1 = (x1 * x1 + y1 * y1).sqrt();
        let mut d2 = (x2 * x2 + y2 * y2).sqrt();
        x1 /= d1;
        y1 /= d1;
        x2 /= d2;
        y2 /= d2;
        let a1 = degrees((x1 * x2 + y1 * y2).acos());

        x1 = (self.a.x - self.b.x) as f64;
        y1 = (self.a.y - self.b.y) as f64;
        x2 = (self.c.x - self.b.x) as f64;
        y2 = (self.c.y - self.b.y) as f64;
        d1 = (x1 * x1 + y1 * y1).sqrt();
        d2 = (x2 * x2 + y2 * y2).sqrt();
        x1 /= d1;
        y1 /= d1;
        x2 /= d2;
        y2 /= d2;
        let a2 = degrees((x1 * x2 + y1 * y2).acos());
        let a3 = 180.0 - a1 - a2;

        a1 > min_degree && a2 > min_degree && a3 > min_degree
    }
}

impl Shape for Triangle {
    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        let (w, h) = img.dimensions();
        let lines = self.rasterize(w, h);
        for line in lines {
            line.draw(img, &color);
        }
    }
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let points = vec![self.a, self.b, self.c];
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
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        let x1 = rng.gen_range(0, w as i32);
        let y1 = rng.gen_range(0, h as i32);
        let x2 = x1 + rng.gen_range(0, 31) - 15;
        let y2 = y1 + rng.gen_range(0, 31) - 15;
        let x3 = x1 + rng.gen_range(0, 31) - 15;
        let y3 = y1 + rng.gen_range(0, 31) - 15;

        let mut triangle = Triangle {
            a: Point { x: x1, y: y1 },
            b: Point { x: x2, y: y2 },
            c: Point { x: x3, y: y3 },
        };
        triangle.mutate(w, h, rng);
        triangle
    }
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        let m = 16;
        loop {
            match rng.gen_range(0, 3) {
                0 => {
                    self.a.x = clamp(
                        self.a.x + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.a.y = clamp(
                        self.a.y + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        h as i32 - 1 + m,
                    );
                }
                1 => {
                    self.b.x = clamp(
                        self.b.x + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.b.y = clamp(
                        self.b.y + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        h as i32 - 1 + m,
                    );
                }
                2 => {
                    self.c.x = clamp(
                        self.c.x + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.c.y = clamp(
                        self.c.y + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        h as i32 - 1 + m,
                    );
                }
                _ => unreachable!(),
            }

            if self.valid() {
                break;
            }
        }
    }

    fn to_svg(&self, attr: &str) -> String {
        format!(
            "<polygon {} points=\"{},{} {},{} {},{}\" />",
            attr, self.a.x, self.a.y, self.b.x, self.b.y, self.c.x, self.c.y,
        )
    }
}

impl PurrShape for Triangle {}
