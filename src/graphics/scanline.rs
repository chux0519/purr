use crate::{alpha_compose, clamp};
use crate::{Rgba, RgbaImage};

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

