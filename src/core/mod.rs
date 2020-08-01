pub mod algo;

pub use algo::*;

use crate::graphics::Scanline;
use crate::{Rgba, RgbaImage};
use rand::rngs::ThreadRng;
use std::path::Path;

pub struct PurrContext {
    pub w: u32,
    pub h: u32,
    // MultiThread, Arc ?
    pub origin_img: RgbaImage,
    pub current_img: RgbaImage,
    pub lines: Vec<Scanline>, // one vec per thread
    pub rng: ThreadRng,
    pub score: f64,
    pub count: u32,
}

impl PurrContext {
    pub fn new<P: AsRef<Path>>(input: P) -> Self {
        let origin_img = image::open(&input).unwrap().into_rgba();
        let (w, h) = origin_img.dimensions();
        // TODO: use avg bg
        let current_img = origin_img.clone();
        PurrContext {
            w,
            h,
            origin_img,
            current_img,
            lines: Vec::new(),
            rng: rand::thread_rng(),
            score: 0.0,
            count: 0,
        }
    }
}

// TODO: HillModel
pub struct PurrModel {
    pub context: Option<PurrContext>,
    pub n: u32,
    pub m: u32,
    pub age: u32,
    // add a threadpool
}

impl PurrModel {
    pub fn new() -> Self {
        PurrModel {
            context: None,
            n: 1000,
            m: 16,
            age: 100,
        }
    }

    pub fn fit(&mut self) {
        match &self.context {
            None => {}
            Some(_ctx) => {
                // run jobs in a threadpool
            }
        }
    }
}
