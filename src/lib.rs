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
