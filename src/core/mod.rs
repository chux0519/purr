pub mod algo;
pub mod hill_climb;

pub use algo::*;
pub use hill_climb::*;

use crate::graphics::*;
use crate::{Rgba, RgbaImage};
use image::imageops::FilterType;
use image::GenericImageView;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::path::Path;

pub trait PurrShape: Clone + Default + Copy + Shape {}

#[derive(Debug, Clone, Copy)]
pub struct PurrState<T> {
    shape: T,
    color: Rgba<u8>,
    score: f64,
}

impl<T: PurrShape> Default for PurrState<T> {
    fn default() -> Self {
        PurrState {
            score: std::f64::MAX,
            color: Rgba([0, 0, 0, 0]),
            shape: T::default(),
        }
    }
}

pub struct PurrContext {
    pub w: u32,
    pub h: u32,
    pub origin_img: RgbaImage,
    pub current_img: RgbaImage,
    pub lines: Vec<Scanline>, // one vec per thread
    pub rng: SmallRng,
    pub score: f64,
    // TODO: heatmap pos
}

impl PurrContext {
    pub fn new<P: AsRef<Path>>(input: P) -> Self {
        let img = image::open(&input).unwrap();
        let (width, height) = img.dimensions();
        let mut w = 0;
        let mut h = 0;
        let max_len = 600;
        let origin_img = if width > height && width > max_len {
            // scale down to max_len
            w = max_len;
            h = (height as f64 / width as f64 * w as f64) as u32;
            let scaled = img.resize(w, h, FilterType::Triangle);
            let img_rgba = scaled.into_rgba();
            let (new_w, new_h) = img_rgba.dimensions();
            w = new_w;
            h = new_h;
            println!(
                "image too large, resize from {}x{} to {}x{}",
                width, height, w, h
            );
            img_rgba
        } else if height > width && height > max_len {
            h = max_len;
            w = (width as f64 / height as f64 * h as f64) as u32;
            let scaled = img.resize(w, h, FilterType::Triangle);
            let img_rgba = scaled.into_rgba();
            let (new_w, new_h) = img_rgba.dimensions();
            w = new_w;
            h = new_h;
            println!(
                "image too large, resize from {}x{} to {}x{}",
                width, height, w, h
            );
            img_rgba
        } else {
            w = width;
            h = height;
            img.into_rgba()
        };

        // init current_img
        let mut current_img = image::ImageBuffer::new(w, h);
        let color = average_color(&origin_img);
        for y in 0..h {
            for x in 0..w {
                let pixel: &mut Rgba<u8> = current_img.get_pixel_mut(x as u32, y as u32);
                pixel.0 = color.0;
            }
        }

        let score = diff_full(&origin_img, &current_img);

        PurrContext {
            w,
            h,
            origin_img,
            current_img,
            lines: Vec::new(),
            rng: SmallRng::from_entropy(),
            score,
        }
    }
}

pub struct PurrModel<T: PurrShape> {
    pub context: PurrContext,
    pub n: u32,
    pub m: u32,
    pub age: u32,
    marker: std::marker::PhantomData<T>,
}

impl<T: PurrShape> PurrModel<T> {
    pub fn new(context: PurrContext, n: u32, m: u32, age: u32) -> Self {
        PurrModel {
            context,
            n,
            m,
            age,
            marker: std::marker::PhantomData::default(),
        }
    }

    pub fn step(&mut self) -> PurrState<T> {
        best_hill_climb(&mut self.context, self.n, self.m, self.age)
    }

    pub fn add_state(&mut self, state: &PurrState<T>) {
        state
            .shape
            .draw(&mut self.context.current_img, &state.color);
        self.context.score = state.score;
    }
}

pub struct PurrModelRunner<T: PurrShape> {
    pub shape_number: u32,
    pub thread_number: u32,
    pub states: Vec<PurrState<T>>,
    // TODO: heatmap
}

impl<T: PurrShape> Default for PurrModelRunner<T> {
    fn default() -> Self {
        PurrModelRunner {
            shape_number: 100,
            thread_number: 4,
            states: Vec::new(),
        }
    }
}

impl<T: PurrShape> PurrModelRunner<T> {
    pub fn run(&mut self, model: &mut PurrModel<T>) {
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
            let output = format!("out_{}.png", i);
            println!("step: {}, score: {}, output: {}", i, state.score, &output);
            self.dump_img(&model.context.current_img, &output);
            self.states.push(state);
        }
    }

    pub fn dump_img(&self, img: &RgbaImage, out: &str) {
        img.save(out).unwrap();
    }
}

