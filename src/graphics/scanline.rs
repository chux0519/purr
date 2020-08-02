use crate::{Rgba, RgbaImage};

#[derive(Debug, Clone, Copy)]
pub struct Scanline {
    pub y: u32,
    pub x1: u32,
    pub x2: u32,
}

impl Scanline {
    pub fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        for x in self.x1..=self.x2 {
            let pixel: &mut Rgba<u8> = img.get_pixel_mut(x as u32, self.y as u32);
            pixel.0 = color.0;
        }
    }
}
