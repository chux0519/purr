use purr::graphics::*;
use purr::*;

fn main() {
    let width = 800;
    let height = 800;
    let mut img = image::ImageBuffer::new(width, height);
    let color = Rgba([0, 255, 255, 127]);
    let p1 = Point { x: 100, y: 100 };
    let p2 = Point { x: 50, y: 150 };
    let p3 = Point { x: 150, y: 150 };
    let lines = triangle(p1, p2, p3);
    for line in lines {
        line.rasterize(&mut img, &color);
    }

    img.save("out.png").unwrap();
}
