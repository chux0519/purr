use crate::clamp;
use crate::core::PurrShape;
use crate::core::{compute_color, diff_partial_with_color};
use crate::core::{PurrContext, PurrState};
use rand::Rng;

pub fn best_hill_climb<T: PurrShape>(
    ctx: &mut PurrContext,
    n: u32,
    m: u32,
    age: u32,
) -> PurrState<T> {
    let mut best_state = PurrState::default();
    for _ in 0..m {
        let best_rand_state = best_random_step(ctx, n);
        let climb_state = hill_climb(ctx, best_rand_state, age);
        if climb_state.score < best_state.score {
            best_state = climb_state;
        }
    }

    best_state
}

pub fn hill_climb<T: PurrShape>(
    ctx: &mut PurrContext,
    state: PurrState<T>,
    age: u32,
) -> PurrState<T> {
    // copy
    let mut cur_state = state;
    let mut best_state = state;
    // best result
    let mut i = 0;
    loop {
        if i > age {
            // cannot find any better state
            break;
        }
        cur_state.shape.mutate(ctx.w, ctx.h, &mut ctx.rng);
        let lines = cur_state.shape.rasterize(ctx.w, ctx.h);
        let alpha = clamp(ctx.rng.gen_range(-10, 11) as i32 + ctx.alpha as i32, 1, 255);
        if lines.is_empty() {
            cur_state = state;
            continue;
        }
        assert!(!lines.is_empty());
        {
            let cur = ctx.current_img.read().unwrap();
            cur_state.color = compute_color(&ctx.origin_img, &cur, &lines, alpha as u8);
            cur_state.score =
                diff_partial_with_color(&ctx.origin_img, &cur, &lines, ctx.score, cur_state.color);
        }

        if cur_state.score < best_state.score {
            // find a new, better state
            best_state = cur_state;
            i = 0;
        } else {
            // undo, restore to last best state
            cur_state = best_state;
            i += 1;
        }
    }
    best_state
}

pub fn best_random_step<T: PurrShape>(ctx: &mut PurrContext, n: u32) -> PurrState<T> {
    let mut best_state = PurrState::default();
    for _ in 0..n {
        let state = random_step(ctx);
        if state.score < best_state.score {
            best_state = state;
        }
    }
    best_state
}
pub fn random_step<T: PurrShape>(ctx: &mut PurrContext) -> PurrState<T> {
    // random generate triangle
    let t = T::random(ctx.w, ctx.h, &mut ctx.rng);
    let lines = t.rasterize(ctx.w, ctx.h);
    if lines.is_empty() {
        return PurrState::default();
    }
    assert!(!lines.is_empty());
    let cur = ctx.current_img.read().unwrap();
    let color = compute_color(&ctx.origin_img, &cur, &lines, ctx.alpha);
    let score = diff_partial_with_color(&ctx.origin_img, &cur, &lines, ctx.score, color);

    PurrState {
        shape: t,
        score,
        color,
    }
}
