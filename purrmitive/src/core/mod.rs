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
use dyn_fmt::AsStrFormatExt;
use gif::{Encoder, Frame, Repeat, SetParameter};
use image::imageops::FilterType;
use image::GenericImageView;
use log::{debug, info};
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
    pub shape: T,
    pub color: Rgba<u8>,
    pub score: f64,
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

impl<T: PurrShape> PurrState<T> {
    fn new(score: f64) -> Self {
        PurrState {
            score,
            color: Rgba([0, 0, 0, 0]),
            shape: T::default(),
        }
    }
    fn to_svg(&self) -> String {
        let attr = format!(
            "fill=\"#{:02X}{:02X}{:02X}\" fill-opacity=\"{}\"",
            self.color.0[0],
            self.color.0[1],
            self.color.0[2],
            self.color.0[3] as f64 / 255.0
        );
        self.shape.to_svg(&attr)
    }
}

#[derive(Clone, Debug)]
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
    pub fn new<P: AsRef<Path>>(
        input: P,
        input_size: u32,
        output_size: u32,
        alpha: u8,
        bg: Option<Rgba<u8>>,
    ) -> Self {
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
            debug!(
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
            debug!(
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

        let color = match bg {
            Some(c) => c,
            None => average_color(&origin_img),
        };

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

#[derive(Clone, Debug)]
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
    pub fn reset(&mut self, context: PurrContext, n: u32, m: u32, age: u32) {
        self.context = context;
        self.n = n;
        self.m = m;
        self.age = age;
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
    pub on_step: Option<Box<dyn FnMut(usize, PurrState<T>) + Sync + Send>>,
}

pub trait PurrModelRunner {
    type M;
    fn init(&mut self, model: &mut Self::M);
    fn step(&mut self, model: &mut Self::M);
    fn stop(&mut self);
    fn run(&mut self, model: &mut Self::M, score: f64);
    fn get_svg(&self, context: &PurrContext, idx: usize) -> String;
    fn save(&self, context: &PurrContext, output: &str);
    fn get_last_shape(&self) -> String;
}

impl<T: PurrShape> Default for PurrMultiThreadRunner<T> {
    fn default() -> Self {
        PurrMultiThreadRunner {
            shape_number: 100,
            thread_number: 4,
            states: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
            on_step: None,
        }
    }
}

impl<T: 'static + PurrShape> PurrModelRunner for PurrMultiThreadRunner<T> {
    type M = PurrHillClimbModel;
    fn init(&mut self, model: &mut Self::M) {
        // stop all threads first
        self.stop();
        self.states.clear();
        // new pool
        if self.txs.is_empty() && self.rxs.is_empty() {
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
        }
    }

    fn step(&mut self, model: &mut Self::M) {
        if self.txs.is_empty() || self.rxs.is_empty() {
            return;
        }

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

        // update main thread
        model.add_state(&best_state);
        self.states.push(best_state);

        match &mut self.on_step {
            None => {}
            Some(f) => f(self.states.len(), best_state),
        }

        // update worker threads
        for tx in &self.txs {
            tx.send(PurrWorkerCmd::UpdateScore(model.context.score))
                .unwrap();
        }
    }

    fn stop(&mut self) {
        // stop workers
        for tx in &self.txs {
            tx.send(PurrWorkerCmd::End).unwrap();
        }
        self.rxs.clear();
        self.txs.clear();
        // pool.join();
    }

    fn run(&mut self, model: &mut Self::M, score: f64) {
        self.init(model);

        if score > 0.0 && score < 1.0 {
            loop {
                self.step(model);
                if model.context.score <= score {
                    break;
                }
            }
        } else {
            for _ in 0..self.shape_number {
                self.step(model);
            }
        }

        self.stop();
    }

    fn get_svg(&self, context: &PurrContext, idx: usize) -> String {
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

        for i in 0..=idx {
            if i >= self.states.len() {
                break;
            }
            output += &self.states[i].to_svg();
        }

        output += "</g>";
        output += "</svg>";
        output
    }

    fn get_last_shape(&self) -> String {
        match self.states.last() {
            Some(s) => s.to_svg(),
            None => "".to_string(),
        }
    }

    fn save(&self, context: &PurrContext, output: &str) {
        // save result
        let suffix = Path::new(output)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap_or("png");
        let should_format = output.find("{").is_some();
        let save_frames = should_format && suffix != "gif";
        for i in 0..self.states.len() {
            let last = i == self.states.len() - 1;
            if last || save_frames {
                let outfile = if should_format {
                    //output.to_string()
                    output.format(&[i + 1])
                } else {
                    output.to_string()
                };
                match suffix {
                    "svg" => {
                        let mut out = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(outfile)
                            .unwrap();
                        out.write_all(self.get_svg(context, i).as_bytes()).unwrap();
                    }
                    "gif" => {
                        let out = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(output)
                            .unwrap();

                        let mut encoder =
                            Encoder::new(out, context.w as u16, context.h as u16, &[0; 0]).unwrap();
                        encoder.set(Repeat::Infinite).unwrap();

                        for n in 0..self.shape_number {
                            let frame = self.get_svg(context, n as usize);
                            info!("exporting {} frame", n + 1);
                            let svg = nsvg::parse_str(&frame, nsvg::Units::Pixel, 96.0).unwrap();
                            let (width, height, mut raw) =
                                svg.rasterize_to_raw_rgba(context.scale).unwrap();
                            // let img = image::RgbaImage::from_raw(width, height, raw).unwrap();
                            let frame = Frame::from_rgba(width as u16, height as u16, &mut raw);
                            encoder.write_frame(&frame).unwrap();
                        }

                        // save final result then
                        let svg_str = self.get_svg(context, i);
                        let img = rasterize_svg(&svg_str, context.scale);
                        let final_res = format!("{}.png", output);
                        img.save(&final_res).unwrap();
                        debug!("gif result saved to {}", final_res);
                    }
                    _ => {
                        // generate svg, then rasterize it
                        // for anti-aliasing
                        let svg_str = self.get_svg(context, i);
                        let img = rasterize_svg(&svg_str, context.scale);
                        img.save(outfile).unwrap();
                    }
                }
            }
        }
    }
}

impl<T: 'static + PurrShape> PurrMultiThreadRunner<T> {
    pub fn new(
        shape_number: u32,
        thread_number: u32,
        on_step: Option<Box<dyn FnMut(usize, PurrState<T>) + Sync + Send>>,
    ) -> Self {
        PurrMultiThreadRunner {
            shape_number,
            thread_number,
            states: Vec::new(),
            rxs: Vec::new(),
            txs: Vec::new(),
            on_step,
        }
    }
}

pub fn rasterize_svg(svg_str: &str, scale: f32) -> RgbaImage {
    let svg = nsvg::parse_str(&svg_str, nsvg::Units::Pixel, 96.0).unwrap();
    let (width, height, raw) = svg.rasterize_to_raw_rgba(scale).unwrap();
    image::RgbaImage::from_raw(width, height, raw).unwrap()
}

#[macro_export]
macro_rules! mt_runner {
    ($x: ty, $shape_number: expr, $thread_number: expr, $cb_creator: expr) => {{
        let cb = Some($cb_creator());
        Box::new(PurrMultiThreadRunner::<$x>::new(
            $shape_number,
            $thread_number,
            cb,
        ))
    }};
}

#[macro_export]
macro_rules! model_runner {
    ($mode: expr, $sn: expr, $tn: expr, $cb_creator: expr) => {{
        let runner: Box<dyn PurrModelRunner<M = PurrHillClimbModel> + Sync + Send> = match $mode {
            0 => mt_runner!(Combo, $sn, $tn, $cb_creator),
            1 => mt_runner!(Triangle, $sn, $tn, $cb_creator),
            2 => mt_runner!(Rectangle, $sn, $tn, $cb_creator),
            3 => mt_runner!(Ellipse, $sn, $tn, $cb_creator),
            4 => mt_runner!(Circle, $sn, $tn, $cb_creator),
            5 => mt_runner!(RotatedRectangle, $sn, $tn, $cb_creator),
            6 => mt_runner!(Quadratic, $sn, $tn, $cb_creator),
            7 => mt_runner!(RotatedEllipse, $sn, $tn, $cb_creator),
            8 => mt_runner!(Polygon, $sn, $tn, $cb_creator),
            _ => {
                error!("unsupported mode {}", $mode);
                unreachable!()
            }
        };
        runner
    }};
}
