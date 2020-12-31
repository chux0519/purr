use purrmitive::core::*;
use purrmitive::graphics::*;
use purrmitive::*;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use env_logger::Builder;
use log::{error, info, LevelFilter};
use once_cell::sync::OnceCell;

static mut RUNNER: OnceCell<Box<dyn PurrModelRunner<M = PurrHillClimbModel> + Send + Sync>> =
    OnceCell::new();

static mut MODEL: OnceCell<PurrHillClimbModel> = OnceCell::new();

#[repr(C)]
pub struct PurrmitiveParam {
    pub alpha: u8,
    pub mode: i32,
    pub resize: u32,
    pub size: u32,
    pub count: u32,
    pub input: *const c_char,
}

#[repr(C)]
pub struct PurrmitiveColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

// TODO: get this info to Qt side, and do the svg concat
#[repr(C)]
pub struct PurrmitiveContextInfo {
    pub w: u32,
    pub h: u32,
    pub scale: f32,
    pub score: f64,
}

fn create_cb<T: PurrShape + std::fmt::Debug>() -> Box<dyn FnMut(PurrState<T>) + Send + Sync> {
    let mut step = 1;
    Box::new(move |x| {
        info!("step {}: {:?}", step, x);
        step += 1;
    })
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_set_verbose(verbose: i32) {
    let mut logger_builder = Builder::new();
    let level = match verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3 | _ => LevelFilter::Trace,
    };
    logger_builder.filter_level(level);
    logger_builder.init();
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_init(param: *const PurrmitiveParam) {
    info!("purrmitive_init");
    let input = CStr::from_ptr((*param).input)
        .to_string_lossy()
        .into_owned();
    info!("input: {}", input);
    let ctx = PurrContext::new(input, (*param).resize, (*param).size, (*param).alpha, None);
    match MODEL.get_mut() {
        Some(m) => {
            // reset
            info!("reset model");
            m.reset(ctx, 1000, 16, 100);
        }
        None => {
            let model = PurrHillClimbModel::new(ctx, 1000, 16, 100);
            match MODEL.set(model) {
                Ok(()) => {}
                Err(_) => error!("Failed to create model!"),
            };
        }
    }

    // if not runner, new one
    match RUNNER.get() {
        Some(_) => {}
        None => {
            let runner = model_runner!(
                (*param).mode,
                (*param).count,
                num_cpus::get() as u32,
                create_cb
            );
            match RUNNER.set(runner) {
                Ok(()) => {}
                Err(_) => error!("Failed to create runner!"),
            };
        }
    }
    match RUNNER.get_mut() {
        Some(r) => {
            // init or reinit
            info!("init/reinit runner");
            r.init(MODEL.get_mut().unwrap());
        }
        None => error!("Failed to init runner!"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_step() {
    match RUNNER.get_mut() {
        Some(r) => match MODEL.get_mut() {
            Some(m) => {
                r.step(m);
            }
            None => {
                error!("Failed to step: Model not found!")
            }
        },
        None => {
            error!("Failed to step: Runner not found!")
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_stop() {
    match RUNNER.get_mut() {
        Some(r) => {
            r.stop();
        }
        None => {
            error!("Failed to stop: Runner not found!")
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_get_bg() -> PurrmitiveColor {
    match MODEL.get() {
        Some(m) => PurrmitiveColor {
            r: m.context.bg.0[0],
            g: m.context.bg.0[1],
            b: m.context.bg.0[2],
            a: m.context.bg.0[3],
        },
        None => {
            error!("No bg color: Model is not found!");
            PurrmitiveColor {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn purrmitive_get_ctx_info() -> PurrmitiveContextInfo {
    match MODEL.get() {
        Some(m) => PurrmitiveContextInfo {
            w: m.context.w,
            h: m.context.h,
            scale: m.context.scale,
            score: m.context.score,
        },
        None => {
            error!("No context info: Model is not found!");
            PurrmitiveContextInfo {
                w: 0,
                h: 0,
                scale: 0.0,
                score: 0.0,
            }
        }
    }
}

#[no_mangle]
// this function will return ownership of the C str, should be freed later
pub unsafe extern "C" fn purrmitive_get_last_shape() -> *mut c_char {
    match RUNNER.get() {
        Some(r) => CString::new(r.get_last_shape()).unwrap().into_raw(),
        None => {
            error!("No frame found: Runner is not found");
            CString::new("").unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn purrmitive_free_str(s: *mut c_char) {
    unsafe {
        if s.is_null() {
            return;
        }
        CString::from_raw(s)
    };
}
