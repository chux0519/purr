use crate::clamp;
use crate::core::PurrShape;
use crate::graphics::bresenham::{rasterize_quad_bezier, rasterize_quad_rational_bezier};
use crate::graphics::{Point, Scanline, Shape};
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};
use rand_distr::StandardNormal;

#[derive(Debug, Clone, Copy)]
pub struct Quadratic {
    pub p0: Point,
    pub p1: Point,
    pub p2: Point,
}

impl Default for Quadratic {
    fn default() -> Self {
        Quadratic {
            p0: Point { x: 0, y: 0 },
            p1: Point { x: 0, y: 0 },
            p2: Point { x: 0, y: 0 },
        }
    }
}

impl Quadratic {
    pub fn valid(&self) -> bool {
        let dx01 = self.p0.x - self.p1.x;
        let dy01 = self.p0.y - self.p1.y;
        let dx12 = self.p1.x - self.p2.x;
        let dy12 = self.p1.y - self.p2.y;
        let dx02 = self.p0.x - self.p2.x;
        let dy02 = self.p0.y - self.p2.y;
        let d01 = dx01 * dx01 + dy01 * dy01;
        let d12 = dx12 * dx12 + dy12 * dy12;
        let d02 = dx02 * dx02 + dy02 * dy02;
        d02 > d01 && d02 > d12
    }
}

impl Shape for Quadratic {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        let px = rng.gen_range(0, w as i32);
        let py = rng.gen_range(0, h as i32);
        let p0 = Point { x: px, y: py };
        let p1 = Point {
            x: px + rng.gen_range(-20, 20),
            y: py + rng.gen_range(-20, 20),
        };
        let p2 = Point {
            x: px + rng.gen_range(-20, 20),
            y: py + rng.gen_range(-20, 20),
        };
        let mut q = Quadratic { p0, p1, p2 };
        q.mutate(w, h, rng);
        q
    }

    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        loop {
            match rng.gen_range(0, 2) {
                0 => {
                    self.p0.x = clamp(
                        self.p0.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        w as i32 - 1,
                    );
                    self.p0.y = clamp(
                        self.p0.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        h as i32 - 1,
                    );
                }
                1 => {
                    self.p1.x = clamp(
                        self.p1.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        w as i32 - 1,
                    );
                    self.p1.y = clamp(
                        self.p1.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        h as i32 - 1,
                    );
                }
                2 => {
                    self.p2.x = clamp(
                        self.p2.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        w as i32 - 1,
                    );
                    self.p2.y = clamp(
                        self.p2.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                        0,
                        h as i32 - 1,
                    );
                }
                _ => unreachable!(),
            }
            if self.valid() {
                break;
            }
        }
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let lines = rasterize_quadratic(self, w, h);
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
        let attr = attr.replace("fill", "stroke");
        format!(
            "<path {} fill=\"none\" d=\"M {} {} Q {} {}, {} {}\" stroke-width=\"{}\" />",
            attr, self.p0.x, self.p0.y, self.p1.x, self.p1.y, self.p2.x, self.p2.y, 1.0
        )
    }
}

fn rasterize_quadratic(q: &Quadratic, w: u32, h: u32) -> Vec<Scanline> {
    let mut ymin = std::i32::MAX;
    let mut ymax = std::i32::MIN;
    for p in &[q.p0, q.p1, q.p2] {
        if p.y < ymin {
            ymin = p.y;
        }
        if p.y > ymax {
            ymax = p.y;
        }
    }
    let range = (ymax - ymin) as usize;

    let mut buf_lhs: Vec<i32> = vec![std::i32::MAX; range + 2];
    let mut buf_rhs: Vec<i32> = vec![std::i32::MIN; range + 2];
    let mut scanlines = Vec::new();

    let x0 = q.p0.x;
    let y0 = q.p0.y;
    let x1 = q.p1.x;
    let y1 = q.p1.y;
    let x2 = q.p2.x;
    let y2 = q.p2.y;

    rasterize_quad_bezier(
        x0,
        y0,
        x1,
        y1,
        x2,
        y2,
        &mut buf_lhs,
        &mut buf_rhs,
        &mut scanlines,
        w,
        h,
        ymin,
    );
    for i in 0..range {
        let y = i as i32 + ymin;
        if y >= 0 && y < h as i32 {
            if buf_lhs[i] >= 0 {
                scanlines.push(Scanline {
                    y: clamp(y as u32, 0, h - 1),
                    x1: clamp(buf_lhs[i], 0, w as i32 - 1) as u32,
                    x2: clamp(buf_rhs[i], 0, w as i32 - 1) as u32,
                });
            }
        }
    }
    scanlines
}

impl PurrShape for Quadratic {}

