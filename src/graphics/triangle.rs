use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{clamp, degrees};
use crate::{Rgba, RgbaImage};
use rand::rngs::SmallRng;
use rand::Rng;
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
        let lines = triangle(self.a, self.b, self.c);
        let mut visible_lines: Vec<Scanline> = lines
            .into_iter()
            .filter(|l| l.x1 <= l.x2 && l.x2 > 0 && l.x1 < w && l.y > 0 && l.y < h)
            .collect();
        for line in &mut visible_lines {
            line.crop(w, h);
        }
        visible_lines
    }
    fn random(w: u32, h: u32, rng: &mut SmallRng) -> Self {
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
    fn mutate(&mut self, w: u32, h: u32, rng: &mut SmallRng) {
        let m = 16;
        loop {
            match rng.gen_range(0, 3) {
                0 => {
                    self.a.x = clamp(
                        self.a.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.a.y = clamp(
                        self.a.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        h as i32 - 1 + m,
                    );
                }
                1 => {
                    self.b.x = clamp(
                        self.b.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.b.y = clamp(
                        self.b.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        h as i32 - 1 + m,
                    );
                }
                2 => {
                    self.c.x = clamp(
                        self.c.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        -m,
                        w as i32 - 1 + m,
                    );
                    self.c.y = clamp(
                        self.c.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
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
}

// old-school way: line sweeping
pub fn triangle(p0: Point, p1: Point, p2: Point) -> Vec<Scanline> {
    let mut lines = Vec::new();
    let mut p0 = p0;
    let mut p1 = p1;
    let mut p2 = p2;
    // sort by y axis, bubble sort
    if p1.y < p0.y {
        std::mem::swap(&mut p0, &mut p1);
    }
    // p0 is the smaller one of left two elements
    // if p2 is samller than p0, then p2 is the samllest one
    if p2.y < p0.y {
        std::mem::swap(&mut p2, &mut p0);
    }
    // then we just reorder remains
    if p2.y < p1.y {
        std::mem::swap(&mut p2, &mut p1);
    }
    assert_eq!(p0.y <= p1.y && p1.y <= p2.y, true);
    // scan the upper triangle
    let total_height = p2.y - p0.y;
    // upper triangle
    for y in p0.y..p1.y {
        let segment_height = p1.y - p0.y + 1;
        let alpha = (y - p0.y) as f64 / total_height as f64;
        let beta = (y - p0.y) as f64 / segment_height as f64;
        let mut a = p0 + (p2 - p0).mul(alpha);
        let mut b = p0 + (p1 - p0).mul(beta);
        if a.x > b.x {
            std::mem::swap(&mut a, &mut b);
        }
        lines.push(Scanline {
            y: y as u32,
            x1: a.x as u32,
            x2: b.x as u32,
        });
    }
    // lower triangle
    for y in p1.y..=p2.y {
        let segment_height = p2.y - p1.y + 1;
        let alpha = (y - p0.y) as f64 / total_height as f64;
        let beta = (y - p1.y) as f64 / segment_height as f64;
        let mut a = p0 + (p2 - p0).mul(alpha);
        let mut b = p1 + (p2 - p1).mul(beta);
        if a.x > b.x {
            std::mem::swap(&mut a, &mut b);
        }
        lines.push(Scanline {
            y: y as u32,
            x1: a.x as u32,
            x2: b.x as u32,
        });
    }

    lines
}

impl PurrShape for Triangle {}
