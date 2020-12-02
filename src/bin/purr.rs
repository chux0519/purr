use clap::{crate_version, App, Arg};
use purrmitive::core::*;
use purrmitive::graphics::*;
use purrmitive::*;

use env_logger::Builder;
use log::{error, info, LevelFilter};

fn create_cb<T: PurrShape + std::fmt::Debug>() -> Box<dyn FnMut(PurrState<T>) + Send + Sync> {
    let mut step = 1;
    Box::new(move |x| {
        info!("{}: {:?}", step, x);
        step += 1;
    })
}

fn main() {
    let matches = App::new("Purr")
        .version(crate_version!())
        .author("Yongsheng Xu")
        .about("Reproducing images with geometric primitives.")
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

    let ctx = PurrContext::new(input, input_size, output_size, alpha, parse_hex_color(bg));
    let mut model = PurrHillClimbModel::new(ctx, 1000, 16, 100);
    let mut runner = model_runner!(shape, shape_number, thread_number, create_cb);
    runner.run(&mut model);
    info!("done, now export to {}", output);
    runner.save(&model.context, output);
}
