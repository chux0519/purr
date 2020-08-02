pub mod algo;
pub mod hill_climb;

pub use algo::*;
pub use hill_climb::*;

use crate::graphics::*;
use crate::{Rgba, RgbaImage};
use rand::rngs::ThreadRng;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct PurrState {
    shape: Triangle,
    color: Rgba<u8>,
    score: f64,
}

impl std::default::Default for PurrState {
    fn default() -> Self {
        PurrState {
            score: std::f64::MAX,
            color: Rgba([0, 0, 0, 0]),
            shape: Triangle {
                a: Point { x: 0, y: 0 },
                b: Point { x: 0, y: 0 },
                c: Point { x: 0, y: 0 },
            },
        }
    }
}

pub struct PurrContext {
    pub w: u32,
    pub h: u32,
    pub origin_img: RgbaImage,
    pub current_img: RgbaImage,
    pub lines: Vec<Scanline>, // one vec per thread
    pub rng: ThreadRng,
    pub score: f64,
    // TODO: heatmap pos
    // TODO: generic
}

impl PurrContext {
    pub fn new<P: AsRef<Path>>(input: P) -> Self {
        let origin_img = image::open(&input).unwrap().into_rgba();
        let (w, h) = origin_img.dimensions();

        // init current_img
        let mut current_img = image::ImageBuffer::new(w, h);
        let mut lines = Vec::new();
        for y in 0..h {
            lines.push(Scanline {
                y,
                x1: 0,
                x2: w - 1,
            });
        }
        let color = compute_color(&origin_img, &current_img, &lines, 255);
        for y in 0..h {
            for x in 0..w {
                let pixel: &mut Rgba<u8> = current_img.get_pixel_mut(x as u32, y as u32);
                pixel.0 = color.0;
            }
        }

        let score = diff_full(&origin_img, &current_img);

        dbg!(&color, &score);

        PurrContext {
            w,
            h,
            origin_img,
            current_img,
            lines: Vec::new(),
            rng: rand::thread_rng(),
            score,
        }
    }
}

pub struct PurrModel {
    pub context: PurrContext,
    pub n: u32,
    pub m: u32,
    pub age: u32,
}

impl PurrModel {
    pub fn new(context: PurrContext) -> Self {
        PurrModel {
            context,
            n: 1000,
            m: 10,
            age: 8,
        }
    }

    pub fn n(&mut self, i: u32) -> &mut Self {
        self.n = i;
        self
    }
    pub fn m(&mut self, i: u32) -> &mut Self {
        self.m = i;
        self
    }
    pub fn age(&mut self, i: u32) -> &mut Self {
        self.age = i;
        self
    }

    pub fn step(&mut self) -> PurrState {
        best_hill_climb(&mut self.context, self.n, self.m, self.age)
    }

    pub fn add_state(&mut self, state: &PurrState) {
        state
            .shape
            .draw(&mut self.context.current_img, &state.color);
        self.context.score = state.score;
    }
}

pub struct PurrModelRunner {
    pub shape_number: u32,
    pub thread_number: u32,
    // TODO: generic
    pub states: Vec<PurrState>,
    // TODO: heatmap
}

impl std::default::Default for PurrModelRunner {
    fn default() -> Self {
        PurrModelRunner {
            shape_number: 100,
            thread_number: 4,
            states: Vec::new(),
        }
    }
}

impl PurrModelRunner {
    pub fn run(&mut self, model: &mut PurrModel) {
        for i in 0..self.shape_number {
            // mpsc
            // let (rx, tx) = mpsc::unbound().unwrap();
            // threadpool.execute(|| {
            //  try sync from global
            //  let result = model.step();
            //  tx.send();
            // });
            // let best_score = max;
            // for i in 0..self.thread_number {
            //  let result = rx.recv();
            //  if result.score < best_score {
            //      best_score = result.score;
            //      best_shape = Some(shape);
            //  }
            //  merge heatmap
            // }
            // draw this frame, update worker threads
            // let shape = model.best_hill_climb();
            // self.shapes.append(shape);
            let state = model.step();
            model.add_state(&state);
            self.dump_img(&model.context.current_img, &format!("out_{}.png", i));
            self.states.push(state);
        }
    }

    pub fn dump_img(&self, img: &RgbaImage, out: &str) {
        img.save(out).unwrap();
    }
}

