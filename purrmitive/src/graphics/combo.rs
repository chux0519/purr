use crate::core::PurrShape;
use crate::graphics::{
    Circle, Ellipse, Polygon, Quadratic, Rectangle, RotatedEllipse, RotatedRectangle, Scanline,
    Shape, Triangle,
};
use crate::{Rgba, RgbaImage};
use rand::{Rng, RngCore, SeedableRng};

#[derive(Debug, Clone, Copy)]
pub enum Combo {
    Triangle(Triangle),
    Ellipse(Ellipse),
    Rectangle(Rectangle),
    RotatedRectangle(RotatedRectangle),
    Circle(Circle),
    Quadratic(Quadratic),
    RotatedEllipse(RotatedEllipse),
    Polygon(Polygon),
}

impl Default for Combo {
    fn default() -> Self {
        Combo::Triangle(Triangle::default())
    }
}

impl Shape for Combo {
    fn random<T: SeedableRng + RngCore>(w: u32, h: u32, rng: &mut T) -> Self {
        match rng.gen_range(0, 8) {
            0 => Combo::Triangle(Triangle::random(w, h, rng)),
            1 => Combo::Ellipse(Ellipse::random(w, h, rng)),
            2 => Combo::Rectangle(Rectangle::random(w, h, rng)),
            3 => Combo::RotatedRectangle(RotatedRectangle::random(w, h, rng)),
            4 => Combo::Circle(Circle::random(w, h, rng)),
            5 => Combo::Quadratic(Quadratic::random(w, h, rng)),
            6 => Combo::RotatedEllipse(RotatedEllipse::random(w, h, rng)),
            7 => Combo::Polygon(Polygon::random(w, h, rng)),
            _ => unreachable!(),
        }
    }

    fn mutate<T: SeedableRng + RngCore>(&mut self, w: u32, h: u32, rng: &mut T) {
        match self {
            Combo::Triangle(s) => s.mutate(w, h, rng),
            Combo::Ellipse(s) => s.mutate(w, h, rng),
            Combo::Rectangle(s) => s.mutate(w, h, rng),
            Combo::RotatedRectangle(s) => s.mutate(w, h, rng),
            Combo::Circle(s) => s.mutate(w, h, rng),
            Combo::Quadratic(s) => s.mutate(w, h, rng),
            Combo::RotatedEllipse(s) => s.mutate(w, h, rng),
            Combo::Polygon(s) => s.mutate(w, h, rng),
        }
    }
    fn rasterize(&self, w: u32, h: u32) -> Vec<Scanline> {
        match self {
            Combo::Triangle(s) => s.rasterize(w, h),
            Combo::Ellipse(s) => s.rasterize(w, h),
            Combo::Rectangle(s) => s.rasterize(w, h),
            Combo::RotatedRectangle(s) => s.rasterize(w, h),
            Combo::Circle(s) => s.rasterize(w, h),
            Combo::Quadratic(s) => s.rasterize(w, h),
            Combo::RotatedEllipse(s) => s.rasterize(w, h),
            Combo::Polygon(s) => s.rasterize(w, h),
        }
    }
    fn draw(&self, img: &mut RgbaImage, color: &Rgba<u8>) {
        match self {
            Combo::Triangle(s) => s.draw(img, color),
            Combo::Ellipse(s) => s.draw(img, color),
            Combo::Rectangle(s) => s.draw(img, color),
            Combo::RotatedRectangle(s) => s.draw(img, color),
            Combo::Circle(s) => s.draw(img, color),
            Combo::Quadratic(s) => s.draw(img, color),
            Combo::RotatedEllipse(s) => s.draw(img, color),
            Combo::Polygon(s) => s.draw(img, color),
        }
    }

    fn to_svg(&self, attr: &str) -> String {
        match self {
            Combo::Triangle(s) => s.to_svg(attr),
            Combo::Ellipse(s) => s.to_svg(attr),
            Combo::Rectangle(s) => s.to_svg(attr),
            Combo::RotatedRectangle(s) => s.to_svg(attr),
            Combo::Circle(s) => s.to_svg(attr),
            Combo::Quadratic(s) => s.to_svg(attr),
            Combo::RotatedEllipse(s) => s.to_svg(attr),
            Combo::Polygon(s) => s.to_svg(attr),
        }
    }
}

impl PurrShape for Combo {}
