// algo implement the core algorithm
use crate::graphics::Scanline;
use crate::{alpha_compose, clamp};
use crate::{Rgba, RgbaImage};

pub fn average_color(img: &RgbaImage) -> Rgba<u8> {
    let (w, h) = img.dimensions();
    let mut r = 0;
    let mut g = 0;
    let mut b = 0;
    for x in 0..w {
        for y in 0..h {
            let pixel: &Rgba<u8> = img.get_pixel(x as u32, y as u32);
            let data = pixel.0;
            r += data[0] as u32;
            g += data[1] as u32;
            b += data[2] as u32;
        }
    }
    r /= w * h;
    g /= w * h;
    b /= w * h;

    Rgba([r as u8, g as u8, b as u8, 255])
}

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

pub fn diff_partial(
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
            total += (dr2 * dr2 + dg2 * dg2 + db2 * db2 + da2 * da2) as u64;
            total -= (dr1 * dr1 + dg1 * dg1 + db1 * db1 + da1 * da1) as u64;
        }
    }
    ((total as f64) / (w * h * 4) as f64).sqrt() / 255.0
}

pub fn diff_full(origin_img: &RgbaImage, current_img: &RgbaImage) -> f64 {
    let mut total = 0;
    let (w, h) = origin_img.dimensions();
    for x in 0..w {
        for y in 0..h {
            let mut pixel: &Rgba<u8> = origin_img.get_pixel(x as u32, y as u32);
            let mut data = pixel.0;
            let or = data[0] as i32;
            let og = data[1] as i32;
            let ob = data[2] as i32;
            let oa = data[3] as i32;

            pixel = current_img.get_pixel(x as u32, y as u32);
            data = pixel.0;

            let cr = data[0] as i32;
            let cg = data[1] as i32;
            let cb = data[2] as i32;
            let ca = data[3] as i32;

            let dr = or - cr;
            let dg = og - cg;
            let db = ob - cb;
            let da = oa - ca;
            total += (dr * dr + dg * dg + db * db + da * da) as u64;
        }
    }
    (total as f64 / (w * h * 4) as f64).sqrt() / 255.0
}

pub fn diff_partial_with_color(
    origin_img: &RgbaImage,
    before_img: &RgbaImage,
    lines: &Vec<Scanline>,
    score: f64,
    color: Rgba<u8>,
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
            let composed_color = alpha_compose(pixel, &color);
            data = pixel.0;
            let br = data[0] as i32;
            let bg = data[1] as i32;
            let bb = data[2] as i32;
            let ba = data[3] as i32;

            let ar = composed_color[0] as i32;
            let ag = composed_color[1] as i32;
            let ab = composed_color[2] as i32;
            let aa = composed_color[3] as i32;

            let dr1 = or - br;
            let dg1 = og - bg;
            let db1 = ob - bb;
            let da1 = oa - ba;
            let dr2 = or - ar;
            let dg2 = og - ag;
            let db2 = ob - ab;
            let da2 = oa - aa;
            total += (dr2 * dr2 + dg2 * dg2 + db2 * db2 + da2 * da2) as u64;
            total -= (dr1 * dr1 + dg1 * dg1 + db1 * db1 + da1 * da1) as u64;
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
                y,
                x1: 0,
                x2: width - 1,
            });
        }

        let c = compute_color(&img, &current_img, &lines, 255);
        assert_eq!(c, color);
    }

    #[test]
    fn test_diff_partial() {
        let width = 100;
        let height = 100;
        let mut img = image::ImageBuffer::new(width, height);
        let before_img = image::ImageBuffer::new(width, height);
        let color = Rgba([255, 0, 0, 255]);
        let mut lines = Vec::new();
        for y in 0..height {
            for x in 0..width {
                let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, y as u32);
                pixel.0 = color.0;
            }
            lines.push(Scanline {
                y,
                x1: 0,
                x2: width - 1,
            });
        }
        let score = diff_partial(&img, &before_img, &before_img, &lines, 0.0);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_diff_full() {
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
                y,
                x1: 0,
                x2: width - 1,
            });
        }
        let score1 = diff_full(&img, &current_img);
        let score2 = diff_full(&img, &img);
        assert_eq!(score1 > 0.0, true);
        assert_eq!(score2, 0.0);
    }
}
