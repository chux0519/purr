use crate::graphics::point::*;
use crate::graphics::scanline::*;

// old-school way: line sweeping
pub fn triangle(p0: Point, p1: Point, p2: Point) -> Vec<Scanline> {
    let mut lines = Vec::new();
    let mut p0 = p0;
    let mut p1 = p1;
    let mut p2 = p2;
    // sort by y axis, bubble sort
    if p1.y < p0.y {
        std::mem::swap(&mut p0, &mut p1);
    }
    // p0 is the smaller one of left two elements
    // if p2 is samller than p0, then p2 is the samllest one
    if p2.y < p0.y {
        std::mem::swap(&mut p2, &mut p0);
    }
    // then we just reorder remains
    if p2.y < p1.y {
        std::mem::swap(&mut p2, &mut p1);
    }
    assert_eq!(p0.y <= p1.y && p1.y <= p2.y, true);
    // scan the upper triangle
    let total_height = p2.y - p0.y;
    // upper triangle
    for y in p0.y..=p1.y {
        let segment_height = p1.y - p0.y + 1;
        let alpha = (y - p0.y) as f64 / total_height as f64;
        let beta = (y - p0.y) as f64 / segment_height as f64;
        let mut A = p0 + (p2 - p0).mul(alpha);
        let mut B = p0 + (p1 - p0).mul(beta);
        if A.x > B.x {
            std::mem::swap(&mut A, &mut B);
        }
        lines.push(Scanline {
            y,
            x1: A.x,
            x2: B.x,
        });
    }
    // lower triangle
    for y in p1.y..=p2.y {
        let segment_height = p2.y - p1.y + 1;
        let alpha = (y - p0.y) as f64 / total_height as f64;
        let beta = (y - p1.y) as f64 / segment_height as f64;
        let mut A = p0 + (p2 - p0).mul(alpha);
        let mut B = p1 + (p2 - p1).mul(beta);
        if A.x > B.x {
            std::mem::swap(&mut A, &mut B);
        }
        lines.push(Scanline {
            y,
            x1: A.x,
            x2: B.x,
        });
    }

    lines
}
