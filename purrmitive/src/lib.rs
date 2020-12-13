pub mod core;
pub mod graphics;

pub use image::*;

fn clamp<T: std::cmp::PartialEq + std::cmp::PartialOrd>(a: T, low: T, high: T) -> T {
    if a < low {
        return low;
    }
    if a > high {
        return high;
    }
    a
}

fn degrees(radians: f64) -> f64 {
    return radians * 180.0 / std::f64::consts::PI;
}

pub fn alpha_compose(bg: &Rgba<u8>, fg: &Rgba<u8>) -> Rgba<u8> {
    let alpha_f = fg.0[3] as f64 / 255.0;
    let r = (bg.0[0] as f64 * (1.0 - alpha_f) + fg.0[0] as f64 * alpha_f) as u8;
    let g = (bg.0[1] as f64 * (1.0 - alpha_f) + fg.0[1] as f64 * alpha_f) as u8;
    let b = (bg.0[2] as f64 * (1.0 - alpha_f) + fg.0[2] as f64 * alpha_f) as u8;
    Rgba([r, g, b, 255])
}
