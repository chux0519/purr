use purr::graphics::*;
use purr::*;

fn main() {
    let width = 800;
    let height = 800;
    let mut img = image::ImageBuffer::new(width, height);
    let color = Rgba([0, 255, 255, 127]);
    let p1 = Point { x: 50, y: 50 };
    let p2 = Point { x: 150, y: 50 };
    let p3 = Point { x: 50, y: 150 };
    // triangle(p1, p2, p3, &mut img, &color);
    let points = vec![p1, p2, p3];
    triangle_barycentric(&points, &mut img, &color);
    img.save("out.png").unwrap();
}
