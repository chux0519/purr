use crate::graphics::point::*;
use crate::{Rgba, RgbaImage};

// old-school way: line sweeping
pub fn triangle(p0: Point, p1: Point, p2: Point, img: &mut RgbaImage, color: &Rgba<u8>) {
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
        for x in A.x..=B.x {
            // img.set(x as usize, y as usize, &color);
            let pixel = img.get_pixel_mut(x as u32, y as u32);
            pixel.0 = color.0;
        }
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
        for x in A.x..=B.x {
            let pixel = img.get_pixel_mut(x as u32, y as u32);
            pixel.0 = color.0;
        }
    }
}

// Barycentric math
pub fn triangle_barycentric(points: &Vec<Point>, img: &mut RgbaImage, color: &Rgba<u8>) {
    // find bound box
    let mut bboxmin = Point {
        x: img.width() as i64 - 1,
        y: img.height() as i64 - 1,
    };
    let mut bboxmax = Point { x: 0, y: 0 };
    let pclamp = Point {
        x: img.width() as i64 - 1,
        y: img.height() as i64 - 1,
    };

    for i in 0..points.len() {
        bboxmin.x = std::cmp::max(0, std::cmp::min(bboxmin.x, points[i].x as i64));
        bboxmin.y = std::cmp::max(0, std::cmp::min(bboxmin.y, points[i].y as i64));
        bboxmax.x = std::cmp::min(pclamp.x, std::cmp::max(bboxmax.x, points[i].x as i64));
        bboxmax.y = std::cmp::min(pclamp.y, std::cmp::max(bboxmax.y, points[i].y as i64));
    }

    // using barycentric coordinates to compute
    let mut p = Point { x: 0, y: 0 };
    for x in bboxmin.x..=bboxmax.x {
        p.x = x;
        for y in bboxmin.y..=bboxmax.y {
            p.y = y;
            let bc: Vec3<f64> = barycentric(points, &p);
            // (x, y, z) is barycentric also named (u, v, w)
            if bc.x < 0f64 || bc.y < 0f64 || bc.z < 0f64 {
                continue;
            }
            let pixel = img.get_pixel_mut(x as u32, y as u32);
            pixel.0 = color.0;
        }
    }
}

fn barycentric(points: &Vec<Point>, p: &Point) -> Vec3<f64> {
    // support triangle only
    assert_eq!(points.len(), 3);
    let v1 = Vec3::<f64> {
        x: points[1].x as f64 - points[0].x as f64,
        y: points[2].x as f64 - points[0].x as f64,
        z: points[0].x as f64 - p.x as f64,
    };
    let v2 = Vec3::<f64> {
        x: points[1].y as f64 - points[0].y as f64,
        y: points[2].y as f64 - points[0].y as f64,
        z: points[0].y as f64 - p.y as f64,
    };

    // (u, v, 1)
    let center = v1 ^ v2;
    if center.z.abs() < 1.0 {
        return Vec3::<f64> {
            x: -1f64,
            y: 1f64,
            z: 1f64,
        };
    }
    // (1 - u -v, u, v)
    Vec3::<f64> {
        x: 1f64 - (center.x + center.y) as f64 / center.z as f64,
        y: center.x as f64 / center.z as f64,
        z: center.y as f64 / center.z as f64,
    }
}
