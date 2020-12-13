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
    for i in 0..=range {
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
    scanlines: &mut Vec<Scanline>,
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
            let y = y0 - ymin;
            if y >= 0 && y < h as i32 {
                if x0 >= 0 {
                    let x = clamp(x0, 0, w as i32 - 1);
                    // used by bezier curve
                    scanlines.push({
                        Scanline {
                            y: clamp(y0, 0, h as i32 - 1) as u32,
                            x1: x as u32,
                            x2: x as u32,
                        }
                    });
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
    scanlines: &mut Vec<Scanline>,
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
            scanlines,
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
            scanlines,
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
    rasterize_quad_bezier_seg(
        x0, y0, x1, y1, x2, y2, buf_lhs, buf_rhs, scanlines, w, h, ymin,
    );
    /* remaining part */ /* remaining part */ /* remaining part */ /* remaining part */
}

// rational
pub fn rasterize_quad_rational_bezier_seg(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    mut w: f64,
    buf_lhs: &mut Vec<i32>,
    buf_rhs: &mut Vec<i32>,
    w_: u32,
    h: u32,
    ymin: i32,
) {
    /* plot a limited rational Bezier segment, squared weight */
    let mut sx = x2 - x1; /* relative values for checks */
    let mut sy = y2 - y1; /* curvature */
    let rhs = sy > 0;
    let lhs = sy < 0;
    let mut dx = (x0 - x2) as f64; /* sign of gradient must not change */
    let mut dy = (y0 - y2) as f64;
    let mut xx = (x0 - x1) as f64;
    let mut yy = (y0 - y1) as f64;
    let mut xy = xx * sy as f64 + yy * sx as f64;
    let mut cur = xx * sy as f64 - yy * sx as f64;
    let mut err;
    assert!(xx * sx as f64 <= 0.0f64 && yy * sy as f64 <= 0.0f64);
    if cur != 0.0f64 && w as f64 > 0.0f64 {
        /* no straight line */
        if (sx * sx + sy * sy) as f64 > xx * xx + yy * yy {
            /* begin with longer part */
            x2 = x0;
            x0 = (x0 as f64 - dx) as i32;
            y2 = y0;
            y0 = (y0 as f64 - dy) as i32;
            cur = -cur
            /* swap P0 P2 */
        }
        /* gradient negates -> algorithm fails */
        xx = 2.0f64 * (4.0f64 * w * sx as f64 * xx + dx * dx); /* differences 2nd degree */
        yy = 2.0f64 * (4.0f64 * w * sy as f64 * yy + dy * dy); /* x step direction */
        sx = if x0 < x2 { 1 } else { -1 }; /* y step direction */
        sy = if y0 < y2 { 1 } else { -1 };
        xy = -2.0f64 * sx as f64 * sy as f64 * (2.0f64 * w * xy + dx * dy);
        if (cur * sx as f64 * sy as f64) < 0.0f64 {
            /* negated curvature? */
            xx = -xx; /* differences 1st degree */
            yy = -yy;
            xy = -xy;
            cur = -cur
        }
        dx = 4.0f64 * w * (x1 - x0) as f64 * sy as f64 * cur + xx / 2.0f64 + xy;
        dy = 4.0f64 * w * (y0 - y1) as f64 * sx as f64 * cur + yy / 2.0f64 + xy;
        if (w) < 0.5f64 && (dy > xy || dx < xy) {
            /* flat ellipse, algorithm fails */
            cur = (w + 1.0f64) / 2.0f64; /* subdivide curve in half */
            w = w.sqrt() as f64; /* plot separately */
            xy = 1.0f64 / (w + 1.0f64); /* error 1.step */
            sx = ((x0 as f64 + 2.0f64 * w * x1 as f64 + x2 as f64) * xy / 2.0f64 + 0.5f64).floor()
                as i32; /* plot curve */
            sy = ((y0 as f64 + 2.0f64 * w * y1 as f64 + y2 as f64) * xy / 2.0f64 + 0.5f64).floor()
                as i32;
            dx = ((w * x1 as f64 + x0 as f64) as f64 * xy + 0.5f64).floor();
            dy = ((y1 as f64 * w + y0 as f64) as f64 * xy + 0.5f64).floor();
            rasterize_quad_rational_bezier_seg(
                x0, y0, dx as i32, dy as i32, sx, sy, cur as f64, buf_lhs, buf_rhs, w_, h, ymin,
            );
            dx = ((w * x1 as f64 + x2 as f64) * xy + 0.5f64).floor();
            dy = ((y1 as f64 * w + y2 as f64) * xy + 0.5f64).floor();
            rasterize_quad_rational_bezier_seg(
                sx, sy, dx as i32, dy as i32, x2, y2, cur as f64, buf_lhs, buf_rhs, w_, h, ymin,
            );
            return;
        }
        err = dx + dy - xy;
        loop {
            let y = y0 - ymin;
            if y >= 0 && y < h as i32 {
                if x0 >= 0 {
                    let x = clamp(x0, 0, w_ as i32 - 1);
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
            /* x step */
            if x0 == x2 && y0 == y2 {
                return;
            } /* last pixel -> curve finished */
            x1 = (2.0 * err > dy) as i32; /* save value for test of x step */
            y1 = (2.0 * (err + yy) < -dy) as i32; /* y step */
            if 2.0 * err < dx || y1 != 0 {
                y0 += sy;
                dy += xy;
                dx += xx;
                err += dx
            }
            if 2.0 * err > dx || x1 != 0 {
                x0 += sx;
                dx += xy;
                dy += yy;
                err += dy
            }
            if !(dy <= xy && dx >= xy) {
                break;
            }
        }
    }
    /* plot remaining needle to end */
    let p0 = Point { x: x0, y: y0 };
    let p2 = Point { x: x2, y: y2 };
    rasterize_line(&p0, &p2, buf_lhs, buf_rhs, w_, h, ymin);
}

pub fn rasterize_quad_rational_bezier(
    mut x0: i32,
    mut y0: i32,
    mut x1: i32,
    mut y1: i32,
    mut x2: i32,
    mut y2: i32,
    mut w: f64,
    buf_lhs: &mut Vec<i32>,
    buf_rhs: &mut Vec<i32>,
    w_: u32,
    h: u32,
    ymin: i32,
) {
    /* plot any quadratic rational Bezier curve */
    let mut x = x0 - 2 * x1 + x2;
    let mut y = y0 - 2 * y1 + y2;
    let mut xx = (x0 - x1) as f64;
    let mut yy = (y0 - y1) as f64;
    let mut ww;
    let mut t;
    let mut q;
    assert!(w >= 0.0f64);
    if xx * (x2 - x1) as f64 > 0.0 {
        /* horizontal cut at P4? */
        if yy * (y2 - y1) as f64 > 0.0 {
            /* now horizontal cut at P4 comes first */
            /* vertical cut at P6 too? */
            if (xx * y as f64).abs() > (yy * x as f64).abs() {
                /* which first? */
                x0 = x2;
                x2 = (xx + x1 as f64) as i32;
                y0 = y2;
                y2 = (yy + y1 as f64) as i32
                /* swap points */
            }
        }
        if x0 == x2 || w == 1.0f64 {
            t = (x0 - x1) as f64 / x as f64
        } else {
            /* P0 = P4, P1 = P8 */
            /* non-rational or rational case */
            q = (4.0f64 * w as f64 * w as f64 * (x0 - x1) as f64 * (x2 - x1) as f64
                + ((x2 - x0) * (x2 - x0)) as f64)
                .sqrt();
            if x1 < x0 {
                q = -q
            }
            t = (2.0f64 * w * (x0 - x1) as f64 - x0 as f64 + x2 as f64 + q)
                / (2.0f64 * (1.0f64 - w) * (x2 - x0) as f64)
            /* t at P4 */
        } /* sub-divide at t */
        q = 1.0f64 / (2.0f64 * t * (1.0f64 - t) * (w - 1.0f64) + 1.0f64); /* = P4 */
        xx = (t * t * (x0 as f64 - 2.0f64 * w * x1 as f64 + x2 as f64)
            + 2.0f64 * t * (w * x1 as f64 - x0 as f64)
            + x0 as f64)
            * q; /* squared weight P3 */
        yy = (t * t * (y0 as f64 - 2.0f64 * w * y1 as f64 + y2 as f64)
            + 2.0f64 * t * (w * y1 as f64 - y0 as f64)
            + y0 as f64)
            * q; /* weight P8 */
        ww = t * (w - 1.0f64) + 1.0f64; /* P4 */
        ww *= ww * q; /* intersect P3 | P0 P1 */
        w = ((1.0f64 - t) * (w as f64 - 1.0f64) + 1.0f64) * q.sqrt(); /* intersect P4 | P1 P2 */
        x = (xx + 0.5f64).floor() as i32;
        y = (yy + 0.5f64).floor() as i32;
        yy = (xx - x0 as f64) * (y1 - y0) as f64 / (x1 - x0) as f64 + y0 as f64;
        rasterize_quad_rational_bezier_seg(
            x0,
            y0,
            x,
            (yy + 0.5f64).floor() as i32,
            x,
            y,
            ww,
            buf_lhs,
            buf_rhs,
            w_,
            h,
            ymin,
        );
        yy = (xx - x2 as f64) * (y1 - y2) as f64 / (x1 - x2) as f64 + y2 as f64;
        y1 = (yy + 0.5f64).floor() as i32;
        x1 = x;
        x0 = x1;
        y0 = y
    }
    if (y0 - y1) * (y2 - y1) > 0 {
        /* vertical cut at P6? */
        if y0 == y2 || w == 1.0f64 {
            t = (y0 - y1) as f64 / (y0 as f64 - 2.0f64 * y1 as f64 + y2 as f64)
        } else {
            /* non-rational or rational case */
            q = (4.0f64 * w * w * (y0 - y1) as f64 * (y2 - y1) as f64
                + ((y2 - y0) * (y2 - y0)) as f64)
                .sqrt();
            if y1 < y0 {
                q = -q
            }
            t = (2.0f64 * w * (y0 - y1) as f64 - y0 as f64 + y2 as f64 + q)
                / (2.0f64 * (1.0f64 - w) * (y2 - y0) as f64)
            /* t at P6 */
        }
        /* P0 = P6, P1 = P7 */
        q = 1.0f64 / (2.0f64 * t * (1.0f64 - t) * (w - 1.0f64) + 1.0f64); /* sub-divide at t */
        xx = (t * t * (x0 as f64 - 2.0f64 * w * x1 as f64 + x2 as f64)
            + 2.0f64 * t * (w * x1 as f64 - x0 as f64) as f64
            + x0 as f64)
            * q; /* = P6 */
        yy = (t * t * (y0 as f64 - 2.0f64 * w * y1 as f64 + y2 as f64)
            + 2.0f64 * t * (w * y1 as f64 - y0 as f64) as f64
            + y0 as f64)
            * q; /* squared weight P5 */
        ww = t * (w - 1.0f64) + 1.0f64; /* weight P7 */
        ww *= ww * q; /* P6 */
        w = ((1.0f64 - t) * (w - 1.0f64) + 1.0f64) * q.sqrt(); /* intersect P6 | P0 P1 */
        x = (xx + 0.5f64).floor() as i32; /* intersect P7 | P1 P2 */
        y = (yy + 0.5f64).floor() as i32;
        xx = (x1 - x0) as f64 * (yy - y0 as f64) / (y1 - y0) as f64 + x0 as f64;
        rasterize_quad_rational_bezier_seg(
            x0,
            y0,
            (xx + 0.5f64).floor() as i32,
            y,
            x,
            y,
            ww,
            buf_lhs,
            buf_rhs,
            w_,
            h,
            ymin,
        );
        xx = (x1 - x2) as f64 * (yy - y2 as f64) / (y1 - y2) as f64 + x2 as f64;
        x1 = (xx + 0.5f64).floor() as i32;
        x0 = x;
        y1 = y;
        y0 = y1
    }
    rasterize_quad_rational_bezier_seg(
        x0,
        y0,
        x1,
        y1,
        x2,
        y2,
        w * w,
        buf_lhs,
        buf_rhs,
        w_,
        h,
        ymin,
    );
    /* remaining */
}

// Ellipse
pub fn rasterize_ellipse(o: &Point, rx: u32, ry: u32) -> Vec<Scanline> {
    let mut lines = Vec::new();
    let mut x = -(rx as i32);
    let mut y = 0;
    let mut e2 = ry as i32;
    let mut dx = (1 + 2 * x) * e2 * e2;
    let mut dy = x * x;
    let mut err = dx + dy;
    let mut skip = false;
    loop {
        if x > 0 {
            break;
        }
        if !skip {
            lines.push(Scanline {
                y: clamp_to_u32(o.y + y),
                x1: clamp_to_u32(o.x + x),
                x2: clamp_to_u32(o.x - x),
            });
            if y != 0 {
                lines.push(Scanline {
                    y: clamp_to_u32(o.y - y),
                    x1: clamp_to_u32(o.x + x),
                    x2: clamp_to_u32(o.x - x),
                });
            }
        }

        e2 = 2 * err;
        if e2 >= dx {
            x += 1;
            dx += 2 * (ry * ry) as i32;
            err += dx;
            skip = true;
        }
        if e2 <= dy {
            y += 1;
            dy += 2 * (rx * rx) as i32;
            err += dy;
            skip = false;
        }
    }

    loop {
        if y >= ry as i32 {
            break;
        }
        lines.push(Scanline {
            y: clamp_to_u32(o.y + y),
            x1: clamp_to_u32(o.x),
            x2: clamp_to_u32(o.x),
        });
        lines.push(Scanline {
            y: clamp_to_u32(o.y - y),
            x1: clamp_to_u32(o.x),
            x2: clamp_to_u32(o.x),
        });
        y += 1;
    }
    lines
}

fn clamp_to_u32(n: i32) -> u32 {
    clamp(n, 0, n) as u32
}

// Rotated Triangle
pub fn rasterize_rotated_ellipse(
    o: &Point,
    rx: u32,
    ry: u32,
    angle: f64,
    w_: u32,
    h: u32,
) -> Vec<Scanline> {
    /* plot ellipse rotated by angle (radian) */
    let mut xd = (rx * rx) as f64;
    let mut yd = (ry * ry) as f64;
    let s = angle.sin();
    let mut zd = (xd - yd) * s; /* ellipse rotation */
    xd = (xd - zd * s).sqrt();
    yd = (yd + zd * s).sqrt(); /* surrounding rectangle */
    let a = xd + 0.5;
    let b = yd + 0.5;
    zd = zd * a * b / (xd * yd); /* scale to integer */

    //TODO: zd == 0.0 could rasterize by rectangle

    rasterize_rotated_ellipse_rect(
        o.x - a as i32,
        o.y - b as i32,
        o.x + a as i32,
        o.y + b as i32,
        4.0 * zd * angle.cos(),
        w_,
        h,
    )
}

pub fn rasterize_rotated_ellipse_rect(
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    zd: f64,
    w_: u32,
    h: u32,
) -> Vec<Scanline> {
    let mut xd = (x1 - x0) as f64;
    let mut yd = (y1 - y0) as f64;
    let mut w = xd * yd;

    if w != 0.0 {
        w = (w - zd) / (w + w);
    } /* squared weight of P1 */

    if !(w <= 1.0 && w >= 0.0) {
        return Vec::new();
    }
    assert!(w <= 1.0 && w >= 0.0); /* limit angle to |zd|<=xd*yd */
    xd = (xd * w + 0.5).floor();
    yd = (yd * w + 0.5).floor(); /* snap xe,ye to int */

    let range = (y1 - y0) as usize;

    // init two y axis buffer
    let mut buf_lhs: Vec<i32> = vec![std::i32::MAX; range + 1];
    let mut buf_rhs: Vec<i32> = vec![std::i32::MIN; range + 1];

    rasterize_quad_rational_bezier_seg(
        x0,
        y0 + yd as i32,
        x0,
        y0,
        x0 + xd as i32,
        y0,
        1.0 - w,
        &mut buf_lhs,
        &mut buf_rhs,
        w_,
        h,
        y0,
    );
    rasterize_quad_rational_bezier_seg(
        x0,
        y0 + yd as i32,
        x0,
        y1,
        x1 - xd as i32,
        y1,
        w,
        &mut buf_lhs,
        &mut buf_rhs,
        w_,
        h,
        y0,
    );
    rasterize_quad_rational_bezier_seg(
        x1,
        y1 - yd as i32,
        x1,
        y1,
        x1 - xd as i32,
        y1,
        1.0 - w,
        &mut buf_lhs,
        &mut buf_rhs,
        w_,
        h,
        y0,
    );
    rasterize_quad_rational_bezier_seg(
        x1,
        y1 - yd as i32,
        x1,
        y0,
        x0 + xd as i32,
        y0,
        w,
        &mut buf_lhs,
        &mut buf_rhs,
        w_,
        h,
        y0,
    );

    let mut scanlines = Vec::new();
    for i in 0..=range {
        let y = i as i32 + y0;
        if y >= 0 && y < h as i32 {
            if buf_rhs[i] >= 0 {
                scanlines.push(Scanline {
                    y: clamp(y as u32, 0, h - 1),
                    x1: clamp(buf_lhs[i], 0, w_ as i32 - 1) as u32,
                    x2: clamp(buf_rhs[i], 0, w_ as i32 - 1) as u32,
                });
            }
        }
    }
    scanlines
}
