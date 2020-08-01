use purr::graphics::*;
use purr::*;

fn main() {
    let width = 800;
    let height = 800;
    let mut img = image::ImageBuffer::new(width, height);
    let color = Rgba([0, 255, 255, 127]);
    let triangle = Triangle {
        a: Point { x: 100, y: 100 },
        b: Point { x: 50, y: 150 },
        c: Point { x: 150, y: 150 },
    };
    let lines = triangle.rasterize();
    for line in lines {
        line.draw(&mut img, &color);
    }
    img.save("out.png").unwrap();
    //
    // let mut model = PurrModel::new(path, n, m, age);
    // model.run();
}
