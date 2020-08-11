use crate::clamp;
use crate::graphics::{Point, Scanline};

// rasterize polygon
// points must be clockwise
pub fn rasterize_polygon(points: &Vec<Point>, w: u32, h: u32) -> Vec<Scanline> {
    if points.len() < 3 {
        return Vec::new();
    }
    // get y range
    let mut ymin = std::i32::MAX;
    let mut ymax = std::i32::MIN;
    for p in points {
        if p.y < ymin {
            ymin = p.y;
        }
        if p.y > ymax {
            ymax = p.y;
        }
    }
    let range = (ymax - ymin) as usize;

    // init two y axis buffer
    let mut buf_lhs: Vec<i32> = vec![std::i32::MAX; range + 1];
    let mut buf_rhs: Vec<i32> = vec![std::i32::MIN; range + 1];

    // scan each line
    for i in 0..(points.len() - 1) {
        let j = i + 1;
        // Pi -> Pj
        rasterize_line(
            &points[i],
            &points[j],
            &mut buf_lhs,
            &mut buf_rhs,
            w,
            h,
            ymin,
        );
    }
    rasterize_line(
        &points[points.len() - 1],
        &points[0],
        &mut buf_lhs,
        &mut buf_rhs,
        w,
        h,
        ymin,
    );

    let mut scanlines = Vec::new();
    for i in 0..range {
        let y = i as i32 + ymin;
        if y >= 0 && y < h as i32 {
            if buf_rhs[i] >= 0 {
                scanlines.push(Scanline {
                    y: clamp(y as u32, 0, h - 1),
                    x1: clamp(buf_lhs[i], 0, w as i32 - 1) as u32,
                    x2: clamp(buf_rhs[i], 0, w as i32 - 1) as u32,
                });
            }
        }
    }
    scanlines
}

pub fn rasterize_line(
    p0: &Point,
    p1: &Point,
    buf_lhs: &mut Vec<i32>,
    buf_rhs: &mut Vec<i32>,
    w: u32,
    h: u32,
    ymin: i32,
) {
    let mut x0 = p0.x;
    let mut y0 = p0.y;
    let x1 = p1.x;
    let y1 = p1.y;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let rhs = p1.y - p0.y > 0;
    let lhs = p1.y - p0.y < 0;

    loop {
        // set x1 and x2
        let y = y0 - ymin;
        if y >= 0 && y < h as i32 {
            if x0 >= 0 {
                let x = clamp(x0, 0, w as i32 - 1);
                if rhs {
                    buf_rhs[y as usize] = x;
                } else if lhs {
                    buf_lhs[y as usize] = x;
                } else {
                    buf_lhs[y as usize] = std::cmp::min(x, buf_lhs[y as usize]);
                    buf_rhs[y as usize] = std::cmp::max(x, buf_rhs[y as usize]);
                }
            }
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            err += dx;
            y0 += sy;
        }
    }
}

// quadratic
pub fn rasterize_quad_bezier_seg(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    buf_lhs: &mut Vec<i32>,
    buf_rhs: &mut Vec<i32>,
    w: u32,
    h: u32,
    ymin: i32,
) {
    let mut sx: i32 = x2 - x1; /* relative values for checks */
    let mut sy: i32 = y2 - y1; /* curvature */
    let mut xx: i32 = x0 - x1; /* sign of gradient must not change */
    let mut yy: i32 = y0 - y1;
    let rhs = sy > 0;
    let lhs = sy < 0;

    let mut xy: i32 = 0;
    let mut dx: f64 = 0.;
    let mut dy: f64 = 0.;
    let mut err: f64 = 0.;
    let mut cur: f64 = (xx * sy - yy * sx) as f64;
    assert!(xx * sx <= 0 && yy * sy <= 0);
    if sx * sx + sy * sy > xx * xx + yy * yy {
        /* begin with longer part */
        x2 = x0;
        x0 = sx + x1;
        y2 = y0;
        y0 = sy + y1;
        cur = -cur
        /* swap P0 P2 */
    }
    if cur != 0.0 {
        /* no straight line */
        xx += sx;
        /* gradient negates -> algorithm fails */
        sx = if x0 < x2 { 1 } else { -1 }; /* x step direction */
        xx *= sx; /* y step direction */
        yy += sy; /* differences 2nd degree */
        sy = if y0 < y2 { 1 } else { -1 };
        yy *= sy;
        xy = 2 * xx * yy;
        xx *= xx;
        yy *= yy;
        if (cur * sx as f64 * sy as f64) < 0.0 {
            /* negated curvature? */
            xx = -xx; /* differences 1st degree */
            yy = -yy; /* error 1st step */
            xy = -xy; /* plot curve */
            cur = -cur
        }
        dx = 4.0f64 * sy as f64 * cur * (x1 - x0) as f64 + xx as f64 - xy as f64;
        dy = 4.0f64 * sx as f64 * cur * (y0 - y1) as f64 + yy as f64 - xy as f64;
        xx += xx;
        yy += yy;
        err = dx + dy + xy as f64;
        loop {
            // TODO: setPixel(x0, y0);
            let y = y0 - ymin;
            if y >= 0 && y < h as i32 {
                if x0 >= 0 {
                    let x = clamp(x0, 0, w as i32 - 1);
                    if rhs {
                        buf_rhs[y as usize] = x;
                    } else if lhs {
                        buf_lhs[y as usize] = x;
                    } else {
                        buf_lhs[y as usize] = std::cmp::min(x, buf_lhs[y as usize]);
                        buf_rhs[y as usize] = std::cmp::max(x, buf_rhs[y as usize]);
                    }
                }
            }
            /* y step */
            if x0 == x2 && y0 == y2 {
                return;
            } /* last pixel -> curve finished */
            y1 = (2.0 * err < dx) as i32; /* save value for test of y step */
            if 2.0 * err > dy {
                x0 += sx; /* x step */
                dx -= xy as f64;
                dy += yy as f64;
                err += dy
            }
            if y1 != 0 {
                y0 += sy;
                dy -= xy as f64;
                dx += xx as f64;
                err += dx
            }
            if !(dy < dx) {
                break;
            }
        }
    }
    // TODO: plotLine(x0, y0, x2, y2);
    let p0 = Point { x: x0, y: y0 };
    let p2 = Point { x: x2, y: y2 };
    rasterize_line(&p0, &p2, buf_lhs, buf_rhs, w, h, ymin);
    /* plot remaining part to end */
}

pub fn rasterize_quad_bezier(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    buf_lhs: &mut Vec<i32>,
    buf_rhs: &mut Vec<i32>,
    w: u32,
    h: u32,
    ymin: i32,
) {
    /* plot any quadratic Bezier curve */
    let mut x: i32 = x0 - x1;
    let mut y: i32 = y0 - y1;
    let mut t: f64 = (x0 - 2 as i32 * x1 + x2) as f64;
    let mut r: f64 = 0.;
    if x * (x2 - x1) > 0 {
        /* horizontal cut at P4? */
        if y * (y2 - y1) > 0 {
            /* now horizontal cut at P4 comes first */
            /* vertical cut at P6 too? */
            if ((y0 - 2 * y1 + y2) as f64 / t * x as f64).abs() > y.abs() as f64 {
                /* which first? */
                x0 = x2;
                x2 = x + x1;
                y0 = y2;
                y2 = y + y1
                /* swap points */
            }
        }
        t = (x0 - x1) as f64 / t;
        /* P0 = P4, P1 = P8 */
        r = (1.0 - t) * ((1.0 - t) * y0 as f64 + 2.0f64 * t * y1 as f64) + t * t * y2 as f64; /* By(t=P4) */
        t = (x0 * x2 - x1 * x1) as f64 * t / (x0 - x1) as f64; /* gradient dP4/dx=0 */
        x = (t + 0.5f64).floor() as i32; /* intersect P3 | P0 P1 */
        y = (r + 0.5f64).floor() as i32; /* intersect P4 | P1 P2 */
        r = (y1 - y0) as f64 * (t - x0 as f64) / (x1 - x0) as f64 + y0 as f64;
        rasterize_quad_bezier_seg(
            x0,
            y0,
            x,
            (r + 0.5f64).floor() as i32,
            x,
            y,
            buf_lhs,
            buf_rhs,
            w,
            h,
            ymin,
        );
        r = (y1 - y2) as f64 * (t - x2 as f64) / (x1 - x2) as f64 + y2 as f64;
        x1 = x;
        x0 = x1;
        y0 = y;
        y1 = (r + 0.5f64).floor() as i32
    }
    if (y0 - y1) * (y2 - y1) > 0 {
        /* vertical cut at P6? */
        t = (y0 - 2 as i32 * y1 + y2) as f64;
        t = (y0 - y1) as f64 / t;
        /* P0 = P6, P1 = P7 */
        r = (1.0 - t) * ((1.0 - t) * x0 as f64 + 2.0f64 * t * x1 as f64) + t * t * x2 as f64; /* Bx(t=P6) */
        t = (y0 * y2 - y1 * y1) as f64 * t / (y0 - y1) as f64; /* gradient dP6/dy=0 */
        x = (r + 0.5f64).floor() as i32; /* intersect P6 | P0 P1 */
        y = (t + 0.5f64).floor() as i32; /* intersect P7 | P1 P2 */
        r = (x1 - x0) as f64 * (t - y0 as f64) / (y1 - y0) as f64 + x0 as f64;
        rasterize_quad_bezier_seg(
            x0,
            y0,
            (r + 0.5f64).floor() as i32,
            y,
            x,
            y,
            buf_lhs,
            buf_rhs,
            w,
            h,
            ymin,
        );
        r = (x1 - x2) as f64 * (t - y2 as f64) / (y1 - y2) as f64 + x2 as f64;
        x0 = x;
        x1 = (r + 0.5f64).floor() as i32;
        y1 = y;
        y0 = y1
    }
    rasterize_quad_bezier_seg(x0, y0, x1, y1, x2, y2, buf_lhs, buf_rhs, w, h, ymin);
    /* remaining part */
}
