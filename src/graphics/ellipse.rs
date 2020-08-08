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
pub struct Ellipse {
    pub o: Point,
    pub rx: u32,
    pub ry: u32,
}

impl Default for Ellipse {
    fn default() -> Self {
        Ellipse {
            o: Point { x: 0, y: 0 },
            rx: 0,
            ry: 0,
        }
    }
}

impl Shape for Ellipse {
    fn random(w: u32, h: u32, rng: &mut SmallRng) -> Self {
        let x = rng.gen_range(0, w as i32);
        let y = rng.gen_range(0, h as i32);
        let rx = rng.gen_range(0, 32) + 1;
        let ry = rng.gen_range(0, 32) + 1;

        Ellipse {
            o: Point { x, y },
            rx,
            ry,
        }
    }
    fn mutate(&mut self, w: u32, h: u32, rng: &mut SmallRng) {
        match rng.gen_range(0, 3) {
            0 => {
                self.o.x = clamp(
                    self.o.x + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    0,
                    w as i32 - 1,
                );
                self.o.y = clamp(
                    self.o.y + (16.0 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    0,
                    h as i32 - 1,
                );
            }
            1 => {
                self.rx = clamp(
                    self.rx + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
                    1,
                    w - 1,
                );
            }
            2 => {
                self.ry = clamp(
                    self.ry + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
                    1,
                    h - 1,
                );
            }
            _ => unreachable!(),
        }
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let lines = ellipse(&self.o, self.rx, self.ry);
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
            "<ellipse {} cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" />",
            attr, self.o.x, self.o.y, self.rx, self.ry
        )
    }
}

// bresenham
pub fn ellipse(o: &Point, rx: u32, ry: u32) -> Vec<Scanline> {
    let mut lines = Vec::new();
    let mut x = -(rx as i32);
    let mut y = 0;
    let mut e2 = ry as i32;
    let mut dx = (1 + 2 * x) * e2 * e2;
    let mut dy = x * x;
    let mut err = dx + dy;
    let mut skip = false;
    loop {
        if x > 0 {
            break;
        }
        if !skip {
            lines.push(Scanline {
                y: clamp_to_u32(o.y + y),
                x1: clamp_to_u32(o.x + x),
                x2: clamp_to_u32(o.x - x),
            });
            if y != 0 {
                lines.push(Scanline {
                    y: clamp_to_u32(o.y - y),
                    x1: clamp_to_u32(o.x + x),
                    x2: clamp_to_u32(o.x - x),
                });
            }
        }

        e2 = 2 * err;
        if e2 >= dx {
            x += 1;
            dx += 2 * (ry * ry) as i32;
            err += dx;
            skip = true;
        }
        if e2 <= dy {
            y += 1;
            dy += 2 * (rx * rx) as i32;
            err += dy;
            skip = false;
        }
    }

    loop {
        if y >= ry as i32 {
            break;
        }
        lines.push(Scanline {
            y: clamp_to_u32(o.y + y),
            x1: clamp_to_u32(o.x),
            x2: clamp_to_u32(o.x),
        });
        lines.push(Scanline {
            y: clamp_to_u32(o.y - y),
            x1: clamp_to_u32(o.x),
            x2: clamp_to_u32(o.x),
        });
        y += 1;
    }
    lines
}

fn clamp_to_u32(n: i32) -> u32 {
    clamp(n, 0, n) as u32
}

impl PurrShape for Ellipse {}
