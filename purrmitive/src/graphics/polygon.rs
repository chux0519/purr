use crate::clamp;
use crate::core::PurrShape;
use crate::graphics::point::*;
use crate::graphics::raster::rasterize_polygon;
use crate::graphics::scanline::*;
use crate::graphics::Shape;
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};
use rand_distr::StandardNormal;

#[derive(Debug, Clone, Copy)]
pub struct Polygon {
    // points length fixed to 4 by now
    pub points: [Point; 4],
}

impl Polygon {
    pub fn clockwise(&mut self) {
        // find centroid
        let c_x: i32 = self.points.iter().map(|p| p.x).sum::<i32>() / 4;
        let c_y: i32 = self.points.iter().map(|p| p.y).sum::<i32>() / 4;

        self.points.sort_by(|a, b| {
            if a.x - c_x >= 0 && b.x - c_x < 0 {
                return std::cmp::Ordering::Greater;
            }
            if a.x - c_x < 0 && b.x - c_x >= 0 {
                return std::cmp::Ordering::Less;
            }
            if a.x - c_x == 0 && b.x - c_x == 0 {
                if a.y - c_y >= 0 || b.y - c_y >= 0 {
                    return a.y.cmp(&b.y);
                }
                return b.y.cmp(&a.y);
            }
            let det = (a.x - c_x) * (b.y - c_y) - (b.x - c_x) * (a.y - c_y);
            if det < 0 {
                return std::cmp::Ordering::Greater;
            }
            if det > 0 {
                return std::cmp::Ordering::Less;
            }
            let d1: i32 = (a.x - c_x) * (a.x - c_x) + (a.y - c_y) * (a.y - c_y);
            let d2 = (b.x - c_x) * (b.x - c_x) + (b.y - c_y) * (b.y - c_y);
            d1.cmp(&d2)
        });
    }
}

impl Default for Polygon {
    fn default() -> Self {
        Polygon {
            points: [
                Point { x: 0, y: 0 },
                Point { x: 0, y: 0 },
                Point { x: 0, y: 0 },
                Point { x: 0, y: 0 },
            ],
        }
    }
}

impl Shape for Polygon {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        let mut polygon = Polygon::default();
        let x0 = rng.gen_range(0, w as i32);
        let y0 = rng.gen_range(0, h as i32);
        polygon.points[0].x = x0;
        polygon.points[0].y = y0;
        for i in 1..4 {
            polygon.points[i].x = x0 + rng.gen_range(-20, 20);
            polygon.points[i].y = y0 + rng.gen_range(-20, 20);
        }
        polygon.mutate(w, h, rng);

        polygon
    }
    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        let m = 16;
        match rng.gen_range(0, 4) {
            0 => {
                let i = rng.gen_range(0, 4);
                let j = rng.gen_range(0, 4);
                if i != j {
                    let p = self.points[i];
                    self.points[i] = self.points[j];
                    self.points[j] = p;
                }
            }
            _ => {
                let i = rng.gen_range(0, 4);
                self.points[i].x = clamp(
                    self.points[i].x + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    -m,
                    w as i32 - 1 + m,
                );
                self.points[i].y = clamp(
                    self.points[i].y + (m as f64 * rng.sample::<f64, _>(StandardNormal)) as i32,
                    -m,
                    h as i32 - 1 + m,
                );
            }
        }
        self.clockwise();
    }

    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        let points = self
            .points
            .iter()
            .map(|p| p.clone())
            .collect::<Vec<Point>>();
        let lines = rasterize_polygon(&points, w, h);
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
        let mut p = format!("<polygon {} points=\"", attr);
        let points_str: String = self
            .points
            .iter()
            .map(|p| format!("{},{}", p.x, p.y))
            .collect::<Vec<String>>()
            .join(",");

        p = p + &points_str + "\"/>";

        p
    }
}

impl PurrShape for Polygon {}
