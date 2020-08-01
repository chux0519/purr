use crate::{Rgba, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Scanline {
    pub y: i64,
    pub x1: i64,
    pub x2: i64,
}

impl Scanline {
    pub fn rasterize(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        for x in self.x1..=self.x2 {
            let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, self.y as u32);
            pixel.0 = color.0;
        }
    }
}
