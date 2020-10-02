use clap::{App, Arg};
use purrmitive::core::*;
use purrmitive::graphics::*;

use env_logger::Builder;
use log::error;
use log::LevelFilter;

fn main() {
    let matches = App::new("Purr")
        .version("0.0")
        .author("Yongsheng Xu")
        .about("Rusty Days Hackathon Project")
        .arg(
            Arg::with_name("input")
                .short("i")
                .help("input image")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .help("output image")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .help("number of shapes")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("thread")
                .short("j")
                .help("numebr of threads")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("mode")
                .short("m")
                .help("mode: 0=combo 1=triangle 2=rect 3=ellipse 4=circle 5=rotatedrect 6=beziers 7=rotatedellipse 8=polygon(default 1)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("resize")
                .short("r")
                .help("input size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .help("output size")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("alpha")
                .short("a")
                .help("alpha value")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("the level of verbosity, v/vv/vvv"),
        )
        .arg(
            Arg::with_name("background")
                .short("b")
                .help("starting background color (hex)")
                .takes_value(true),
        )
        .get_matches();
    let mut logger_builder = Builder::new();
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();
    let shape_number = matches.value_of("number").unwrap().parse().unwrap();
    let shape = matches.value_of("mode").unwrap_or("1").parse().unwrap();
    let thread_number = matches
        .value_of("thread")
        .unwrap_or(&num_cpus::get().to_string())
        .parse()
        .unwrap();
    let input_size = matches.value_of("resize").unwrap_or("256").parse().unwrap();
    let output_size = matches.value_of("size").unwrap_or("1024").parse().unwrap();
    let alpha = matches.value_of("alpha").unwrap_or("128").parse().unwrap();
    let bg = matches.value_of("background").unwrap_or("");

    let level = match matches.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3 | _ => LevelFilter::Trace,
    };
    logger_builder.filter_level(level);
    logger_builder.init();

    let model_ctx = PurrContext::new(input, input_size, output_size, alpha, parse_hex_color(bg));
    let mut model_hillclimb = PurrHillClimbModel::new(model_ctx, 1000, 16, 100);
    let mut model_runner: Box<dyn PurrModelRunner<M = PurrHillClimbModel>> = match shape {
        0 => Box::new(PurrMultiThreadRunner::<Combo>::new(
            shape_number,
            thread_number,
        )),
        1 => Box::new(PurrMultiThreadRunner::<Triangle>::new(
            shape_number,
            thread_number,
        )),
        2 => Box::new(PurrMultiThreadRunner::<Rectangle>::new(
            shape_number,
            thread_number,
        )),
        3 => Box::new(PurrMultiThreadRunner::<Ellipse>::new(
            shape_number,
            thread_number,
        )),
        4 => Box::new(PurrMultiThreadRunner::<Circle>::new(
            shape_number,
            thread_number,
        )),
        5 => Box::new(PurrMultiThreadRunner::<RotatedRectangle>::new(
            shape_number,
            thread_number,
        )),
        6 => Box::new(PurrMultiThreadRunner::<Quadratic>::new(
            shape_number,
            thread_number,
        )),
        7 => Box::new(PurrMultiThreadRunner::<RotatedEllipse>::new(
            shape_number,
            thread_number,
        )),
        8 => Box::new(PurrMultiThreadRunner::<Polygon>::new(
            shape_number,
            thread_number,
        )),
        _ => {
            error!("unsupported shape {}", shape);
            unreachable!()
        }
    };
    model_runner.run(&mut model_hillclimb, output);
}
