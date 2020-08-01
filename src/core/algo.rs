// algo implement the core algorithm
use crate::graphics::Scanline;
use crate::{Rgba, RgbaImage};

pub fn compute_color(
    origin_img: &RgbaImage,
    current_img: &RgbaImage,
    lines: &Vec<Scanline>,
    alpha: u8,
) -> Rgba<u8> {
    let mut rsum = 0;
    let mut gsum = 0;
    let mut bsum = 0;
    let mut count = 0;
    let a: i32 = 0x101 * 255 / alpha as i32;

    for line in lines {
        for x in line.x1..=line.x2 {
            let mut pixel: &Rgba<u8> = origin_img.get_pixel(x as u32, line.y as u32);
            let mut data = pixel.0;
            let or = data[0] as i32;
            let og = data[1] as i32;
            let ob = data[2] as i32;

            pixel = current_img.get_pixel(x as u32, line.y as u32);
            data = pixel.0;

            let cr = data[0] as i32;
            let cg = data[1] as i32;
            let cb = data[2] as i32;
            rsum += ((or - cr) * a + cr * 0x101) as i64;
            gsum += ((og - cg) * a + cg * 0x101) as i64;
            bsum += ((ob - cb) * a + cb * 0x101) as i64;
            count += 1;
        }
    }
    assert_eq!(count > 0, true);
    let r = clamp((rsum / count) as i32 >> 8, 0, 255) as u8;
    let g = clamp((gsum / count) as i32 >> 8, 0, 255) as u8;
    let b = clamp((bsum / count) as i32 >> 8, 0, 255) as u8;
    Rgba([r, g, b, alpha])
}

fn clamp<T: std::cmp::PartialEq + std::cmp::PartialOrd>(a: T, low: T, high: T) -> T {
    if a < low {
        return low;
    }
    if a > high {
        return high;
    }
    a
}

pub fn partial_diff(
    origin_img: &RgbaImage,
    before_img: &RgbaImage,
    after_img: &RgbaImage,
    lines: &Vec<Scanline>,
    score: f64,
) -> f64 {
    let (w, h) = origin_img.dimensions();
    let mut total: u64 = ((score * 255.0) * (score * 255.0) * (w * h * 4) as f64) as u64;

    for line in lines {
        for x in line.x1..=line.x2 {
            let mut pixel: &Rgba<u8> = origin_img.get_pixel(x as u32, line.y as u32);
            let mut data = pixel.0;
            let or = data[0] as i32;
            let og = data[1] as i32;
            let ob = data[2] as i32;
            let oa = data[3] as i32;

            pixel = before_img.get_pixel(x as u32, line.y as u32);
            data = pixel.0;
            let br = data[0] as i32;
            let bg = data[1] as i32;
            let bb = data[2] as i32;
            let ba = data[3] as i32;

            pixel = after_img.get_pixel(x as u32, line.y as u32);
            data = pixel.0;
            let ar = data[0] as i32;
            let ag = data[1] as i32;
            let ab = data[2] as i32;
            let aa = data[3] as i32;

            let dr1 = or - br;
            let dg1 = og - bg;
            let db1 = ob - bb;
            let da1 = oa - ba;
            let dr2 = or - ar;
            let dg2 = og - ag;
            let db2 = ob - ab;
            let da2 = oa - aa;
            total -= (dr1 * dr1 + dg1 * dg1 + db1 * db1 + da1 * da1) as u64;
            total += (dr2 * dr2 + dg2 * dg2 + db2 * db2 + da2 * da2) as u64;
        }
    }
    ((total as f64) / (w * h * 4) as f64).sqrt() / 255.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_compute_color() {
        let width = 100;
        let height = 100;
        let mut img = image::ImageBuffer::new(width, height);
        let current_img = image::ImageBuffer::new(width, height);
        let color = Rgba([255, 0, 0, 255]);
        let mut lines = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, y as u32);
                pixel.0 = color.0;
            }
            lines.push(Scanline {
                y: y as i64,
                x1: 0,
                x2: width as i64 - 1,
            });
        }

        let c = compute_color(&img, &current_img, &lines, 255);
        assert_eq!(c, color);
    }
}
