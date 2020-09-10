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
use gif::{Encoder, Frame, Repeat, SetParameter};
use image::imageops::FilterType;
use image::GenericImageView;
use nsvg;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::Write;
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
    pub scale: f32,
    pub origin_img: Arc<RgbaImage>,
    pub current_img: Arc<RwLock<RgbaImage>>,
    pub rng: SmallRng,
    pub score: f64,
    pub bg: Rgba<u8>, // TODO: heatmap pos
    pub alpha: u8,
}

impl PurrContext {
    pub fn new<P: AsRef<Path>>(input: P, input_size: u32, output_size: u32, alpha: u8) -> Self {
        let img = image::open(&input).unwrap();
        let (width, height) = img.dimensions();
        let mut w;
        let mut h;
        let origin_img = if width >= height && width > input_size {
            // scale down to max_size
            w = input_size;
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
        } else if height > width && height > input_size {
            h = input_size;
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
        let scale = output_size as f32 / input_size as f32;

        PurrContext {
            w,
            h,
            scale,
            origin_img: Arc::new(origin_img),
            current_img: Arc::new(RwLock::new(current_img)),
            rng: SmallRng::from_entropy(),
            score,
            bg: color,
            alpha,
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

pub struct PurrMultiThreadRunner<T: PurrShape> {
    pub shape_number: u32,
    pub thread_number: u32,
    pub states: Vec<PurrState<T>>,
    pub rxs: Vec<Receiver<PurrState<T>>>,
    pub txs: Vec<Sender<PurrWorkerCmd>>,
}

pub trait PurrModelRunner {
    type M;
    fn run(&mut self, model: &mut Self::M, output: &str);
    fn to_svg(&self, context: &PurrContext) -> String;
    fn to_frames(&self, context: &PurrContext) -> Vec<String>;
}

impl<T: PurrShape> Default for PurrMultiThreadRunner<T> {
    fn default() -> Self {
        PurrMultiThreadRunner {
            shape_number: 100,
            thread_number: 4,
            states: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
        }
    }
}

impl<T: 'static + PurrShape> PurrModelRunner for PurrMultiThreadRunner<T> {
    type M = PurrHillClimbModel;
    fn run(&mut self, model: &mut Self::M, output: &str) {
        let pool = ThreadPool::new(self.thread_number as usize);
        // spawn workers
        let mut worker_model_m = model.m / self.thread_number;
        if model.m % self.thread_number != 0 {
            worker_model_m += 1;
        }
        for _ in 0..self.thread_number {
            let (cmd_s, cmd_r) = bounded(1);
            let (res_s, res_r) = bounded(1);
            let mut worker_model = model.clone();
            worker_model.m = worker_model_m;
            worker_model.context.rng = SmallRng::from_entropy();
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
        }

        // stop workers
        for tx in &self.txs {
            tx.send(PurrWorkerCmd::End).unwrap();
        }

        pool.join();

        println!("jobs done, now export to {}", output);

        // save result
        {
            let suffix = Path::new(output)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("png");
            match suffix {
                "svg" => {
                    let mut out = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(output)
                        .unwrap();
                    out.write_all(self.to_svg(&model.context).as_bytes())
                        .unwrap();
                }
                "gif" => {
                    let out = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .open(output)
                        .unwrap();
                    let frames = self.to_frames(&model.context);
                    let mut encoder =
                        Encoder::new(out, model.context.w as u16, model.context.h as u16, &[0; 0])
                            .unwrap();
                    encoder.set(Repeat::Infinite).unwrap();

                    for (i, frame_str) in frames.iter().enumerate() {
                        println!("exporting {} frame", i + 1);
                        let svg = nsvg::parse_str(frame_str, nsvg::Units::Pixel, 96.0).unwrap();
                        let (width, height, mut raw) =
                            svg.rasterize_to_raw_rgba(model.context.scale).unwrap();
                        // let img = image::RgbaImage::from_raw(width, height, raw).unwrap();
                        let frame = Frame::from_rgba(width as u16, height as u16, &mut raw);
                        encoder.write_frame(&frame).unwrap();
                    }
                    // save final result then
                    let svg_str = self.to_svg(&model.context);
                    let img = rasterize_svg(&svg_str, model.context.scale);
                    let final_res = format!("{}.png", output);
                    img.save(&final_res).unwrap();
                    println!("final result saved to {}", final_res);
                }
                _ => {
                    // generate svg, then rasterize it
                    // for anti-aliasing
                    let svg_str = self.to_svg(&model.context);
                    let img = rasterize_svg(&svg_str, model.context.scale);
                    img.save(output).unwrap();
                }
            }
        }
    }

    fn to_svg(&self, context: &PurrContext) -> String {
        let mut output = "".to_owned();
        output += &format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">",
            context.w, context.h
        );
        output += &format!(
            "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{:02X}{:02X}{:02X}\"/>",
            context.w, context.h, context.bg.0[0], context.bg.0[1], context.bg.0[2]
        );
        output += "<g transform=\"scale(1) translate(0.5 0.5)\">";

        for s in &self.states {
            let attr = format!(
                "fill=\"#{:02X}{:02X}{:02X}\" fill-opacity=\"{}\"",
                s.color.0[0],
                s.color.0[1],
                s.color.0[2],
                s.color.0[3] as f64 / 255.0
            );
            output += &s.shape.to_svg(&attr);
        }

        output += "</g>";
        output += "</svg>";
        output
    }

    fn to_frames(&self, context: &PurrContext) -> Vec<String> {
        let mut frames = Vec::new();
        let mut head = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">",
            context.w, context.h
        );
        head += &format!(
            "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{:02X}{:02X}{:02X}\"/>",
            context.w, context.h, context.bg.0[0], context.bg.0[1], context.bg.0[2]
        );
        head += "<g transform=\"scale(1) translate(0.5 0.5)\">";
        let tail = "</g></svg>";
        for n in 0..self.shape_number {
            let mut frame = head.clone();
            for i in 0..n {
                let s = &self.states[i as usize];
                let attr = format!(
                    "fill=\"#{:02X}{:02X}{:02X}\" fill-opacity=\"{}\"",
                    s.color.0[0],
                    s.color.0[1],
                    s.color.0[2],
                    s.color.0[3] as f64 / 255.0
                );
                frame += &s.shape.to_svg(&attr);
            }
            frame += tail;
            frames.push(frame);
        }
        frames
    }
}

impl<T: 'static + PurrShape> PurrMultiThreadRunner<T> {
    pub fn new(shape_number: u32, thread_number: u32) -> Self {
        PurrMultiThreadRunner {
            shape_number,
            thread_number,
            states: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
        }
    }
}

pub fn rasterize_svg(svg_str: &str, scale: f32) -> RgbaImage {
    let svg = nsvg::parse_str(&svg_str, nsvg::Units::Pixel, 96.0).unwrap();
    let (width, height, raw) = svg.rasterize_to_raw_rgba(scale).unwrap();
    image::RgbaImage::from_raw(width, height, raw).unwrap()
}
