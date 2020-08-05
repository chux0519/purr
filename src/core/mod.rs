pub mod algo;
pub mod hill_climb;
pub mod worker;

pub use algo::*;
pub use hill_climb::*;
pub use worker::*;

use crate::graphics::*;
use crate::{Rgba, RgbaImage};
use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};
use image::gif::Encoder;
use image::imageops::FilterType;
use image::Frame;
use image::GenericImageView;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Arc, RwLock};
use threadpool::ThreadPool;

pub trait PurrShape: Clone + Default + Copy + Shape + Send {}

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

#[derive(Clone)]
pub struct PurrContext {
    pub w: u32,
    pub h: u32,
    pub origin_img: Arc<RgbaImage>,
    pub current_img: Arc<RwLock<RgbaImage>>,
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
            origin_img: Arc::new(origin_img),
            current_img: Arc::new(RwLock::new(current_img)),
            rng: SmallRng::from_entropy(),
            score,
        }
    }
}

pub trait PurrModel<T: PurrShape> {
    fn step(&mut self) -> PurrState<T>;
    fn add_state(&mut self, state: &PurrState<T>);
}

#[derive(Clone)]
pub struct PurrHillClimbModel {
    pub context: PurrContext,
    pub n: u32,
    pub m: u32,
    pub age: u32,
}

impl PurrHillClimbModel {
    pub fn new(context: PurrContext, n: u32, m: u32, age: u32) -> Self {
        PurrHillClimbModel { context, n, m, age }
    }
}

impl<T: PurrShape> PurrModel<T> for PurrHillClimbModel {
    fn step(&mut self) -> PurrState<T> {
        best_hill_climb(&mut self.context, self.n, self.m, self.age)
    }

    fn add_state(&mut self, state: &PurrState<T>) {
        let mut cur = self.context.current_img.write().unwrap();
        state.shape.draw(&mut cur, &state.color);
        self.context.score = state.score;
    }
}

pub struct PurrModelRunner<T: PurrShape> {
    pub shape_number: u32,
    pub thread_number: u32,
    pub states: Vec<PurrState<T>>,
    pub frames: Vec<Frame>, // TODO: heatmap
    pub rxs: Vec<Receiver<PurrState<T>>>,
    pub txs: Vec<Sender<PurrWorkerCmd>>,
}

pub trait ModelRunner {
    fn run(&mut self, model: &mut PurrHillClimbModel, output: &str);
}

impl<T: PurrShape> Default for PurrModelRunner<T> {
    fn default() -> Self {
        PurrModelRunner {
            shape_number: 100,
            thread_number: 4,
            states: Vec::new(),
            frames: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
        }
    }
}

impl<T: 'static + PurrShape> ModelRunner for PurrModelRunner<T> {
    fn run(&mut self, model: &mut PurrHillClimbModel, output: &str) {
        let pool = ThreadPool::new(self.thread_number as usize);
        // spawn workers
        let worker_model_m = model.m / self.thread_number;
        for _ in 0..self.thread_number {
            let (cmd_s, cmd_r) = bounded(1);
            let (res_s, res_r) = bounded(1);
            let mut worker_model = model.clone();
            worker_model.m = worker_model_m;
            let mut worker = PurrWorker::new(worker_model, cmd_r, res_s);
            self.txs.push(cmd_s);
            self.rxs.push(res_r);
            pool.execute(move || {
                worker.start();
            });
        }

        for batch in 0..self.shape_number {
            // wake all workers
            for tx in &self.txs {
                tx.send(PurrWorkerCmd::Start).unwrap();
            }

            // wait for result
            let mut best_state = PurrState::default();
            for rx in &self.rxs {
                let state = rx.recv().unwrap();
                if state.score < best_state.score {
                    best_state = state;
                }
            }

            println!(
                "Batch: {}, score {} -> score {}",
                batch + 1,
                model.context.score,
                best_state.score
            );
            // update main thread
            model.add_state(&best_state);
            self.states.push(best_state);

            // update worker threads
            for tx in &self.txs {
                tx.send(PurrWorkerCmd::UpdateScore(model.context.score))
                    .unwrap();
            }

            //self.frames
            //    .push(Frame::new(model.context.current_img.clone()));
        }

        // stop workers
        for tx in &self.txs {
            tx.send(PurrWorkerCmd::End).unwrap();
        }

        pool.join();

        {
            let res = model.context.current_img.read().unwrap();
            dump_img(&res, output);
        }
        //let file_out = OpenOptions::new()
        //    .write(true)
        //    .create(true)
        //    .open("out.gif")
        //    .unwrap();
        //let mut encoder = Encoder::new(file_out);
        //encoder
        //    .encode_frames(self.frames.clone().into_iter())
        //    .unwrap();
    }
}

impl<T: 'static + PurrShape> PurrModelRunner<T> {
    pub fn new(shape_number: u32, thread_number: u32) -> Self {
        PurrModelRunner {
            shape_number,
            thread_number,
            states: Vec::new(),
            frames: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
        }
    }
}

pub fn dump_img(img: &RgbaImage, out: &str) {
    img.save(out).unwrap();
}
