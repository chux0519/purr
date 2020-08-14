use crate::clamp;
use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::raster::{rasterize_ellipse, rasterize_rotated_ellipse};
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};
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

impl Ellipse {
    pub fn mutate_o<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
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

    pub fn mutate_rx<T: SeedableRng + RngCore>(&mut self, w: u32, rng: &mut T) {
        self.rx = clamp(
            self.rx + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
            1,
            w - 1,
        );
    }

    pub fn mutate_ry<T: SeedableRng + RngCore>(&mut self, h: u32, rng: &mut T) {
        self.ry = clamp(
            self.ry + (16.0 * rng.sample::<f64, _>(StandardNormal)) as u32,
            1,
            h - 1,
        );
    }
}

impl Shape for Ellipse {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
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
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        match rng.gen_range(0, 3) {
            0 => {
                self.mutate_o(w, h, rng);
            }
            1 => {
                self.mutate_rx(w, rng);
            }
            2 => {
                self.mutate_ry(h, rng);
            }
            _ => unreachable!(),
        }
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let lines = rasterize_ellipse(&self.o, self.rx, self.ry);
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

impl PurrShape for Ellipse {}

// Rotated

#[derive(Debug, Clone, Copy)]
pub struct RotatedEllipse {
    pub degree: u32,
    pub ellipse: Ellipse,
}

impl Default for RotatedEllipse {
    fn default() -> Self {
        RotatedEllipse {
            degree: 0,
            ellipse: Ellipse {
                o: Point { x: 0, y: 0 },
                rx: 0,
                ry: 0,
            },
        }
    }
}

impl Shape for RotatedEllipse {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        RotatedEllipse {
            degree: rng.gen_range(0, 360),
            ellipse: Ellipse::random(w, h, rng),
        }
    }
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        match rng.gen_range(0, 4) {
            0 => {
                self.ellipse.mutate_o(w, h, rng);
            }
            1 => {
                self.ellipse.mutate_rx(w, rng);
            }
            2 => {
                self.ellipse.mutate_ry(h, rng);
            }
            3 => {
                // mutate degree
                self.degree += (32.0 * rng.sample::<f64, _>(StandardNormal)) as u32;
            }
            _ => unreachable!(),
        }
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let angle = self.degree as f64 * std::f64::consts::PI / 180.0;
        let lines = rasterize_rotated_ellipse(
            &self.ellipse.o,
            self.ellipse.rx,
            self.ellipse.ry,
            angle,
            w,
            h,
        );
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
		"<g transform=\"translate({} {}) rotate({}) scale({} {})\"><ellipse {} cx=\"0\" cy=\"0\" rx=\"1\" ry=\"1\" /></g>",
            self.ellipse.o.x, self.ellipse.o.y, self.degree, self.ellipse.rx, self.ellipse.ry, attr
        )
    }
}

impl PurrShape for RotatedEllipse {}

