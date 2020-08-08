use crate::graphics::Point;
use crate::{alpha_compose, clamp, Rgba, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Scanline {
    pub y: u32,
    pub x1: u32,
    pub x2: u32,
}

impl Scanline {
    pub fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        assert!(self.x1 <= self.x2);
        for x in self.x1..=self.x2 {
            let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, self.y as u32);
            // (foreground.r * alpha) + (background.r * (1.0 - alpha));
            let c = alpha_compose(pixel, color);
            pixel.0 = c.0;
        }
    }

    pub fn crop(&mut self, w: u32, h: u32) {
        self.y = clamp(self.y, 0, h - 1);
        self.x1 = clamp(self.x1, 0, w - 1);
        self.x2 = clamp(self.x2, 0, w - 1);
    }
}

// rasterize polygon
// points must be clockwise
pub fn scan_polygon(points: &Vec<Point>, w: u32, h: u32) -> Vec<Scanline> {
    if points.len() < 3 {
        return Vec::new();
    }
    // get y range
    let mut ymin = std::i32::MAX;
    let mut ymax = std::i32::MIN;
    for p in points {
        if p.y < ymin {
            ymin = p.y;
        }
        if p.y > ymax {
            ymax = p.y;
        }
    }
    let range = (ymax - ymin) as usize;

    // init two y axis buffer
    let mut buf_x1: Vec<i32> = vec![std::i32::MAX; range + 1];
    let mut buf_x2: Vec<i32> = vec![std::i32::MIN; range + 1];

    // scan each line
    for i in 0..(points.len() - 1) {
        let j = i + 1;
        // Pi -> Pj
        rasterize_line(&points[i], &points[j], &mut buf_x1, &mut buf_x2, w, h, ymin);
    }
    rasterize_line(
        &points[points.len() - 1],
        &points[0],
        &mut buf_x1,
        &mut buf_x2,
        w,
        h,
        ymin,
    );

    let mut scanlines = Vec::new();
    for i in 0..range {
        let y = i as i32 + ymin;
        if y >= 0 && y < h as i32 {
            if buf_x2[i] >= 0 {
                scanlines.push(Scanline {
                    y: clamp(y as u32, 0, h - 1),
                    x1: clamp(buf_x1[i], 0, w as i32 - 1) as u32,
                    x2: clamp(buf_x2[i], 0, w as i32 - 1) as u32,
                });
            }
        }
    }
    scanlines
}

pub fn rasterize_line(
    p0: &Point,
    p1: &Point,
    buf_x1: &mut Vec<i32>,
    buf_x2: &mut Vec<i32>,
    w: u32,
    h: u32,
    ymin: i32,
) {
    let mut x0 = p0.x;
    let mut y0 = p0.y;
    let x1 = p1.x;
    let y1 = p1.y;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let rhs = p1.y - p0.y > 0;
    let lhs = p1.y - p0.y < 0;

    loop {
        // set x1 and x2
        let y = y0 - ymin;
        if y >= 0 && y < h as i32 {
            if x0 >= 0 {
                let x = clamp(x0, 0, w as i32 - 1);
                if rhs {
                    buf_x2[y as usize] = x;
                } else if lhs {
                    buf_x1[y as usize] = x;
                } else {
                    buf_x1[y as usize] = std::cmp::min(x, buf_x1[y as usize]);
                    buf_x2[y as usize] = std::cmp::max(x, buf_x2[y as usize]);
                }
            }
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            err += dx;
            y0 += sy;
        }
    }
}
