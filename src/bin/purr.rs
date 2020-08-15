use clap::{App, Arg};
use purr::core::*;
use purr::graphics::*;

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
                .help("mode: 0=combo 1=triangle 2=rect 3=ellipse 4=circle 5=rotatedrect 6=beziers 7=rotatedellipse (default 1)")
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
        .get_matches();
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

    let model_ctx = PurrContext::new(input, input_size, output_size);
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
        _ => {
            eprintln!("unsupported shape {}", shape);
            unreachable!()
        }
    };
    model_runner.run(&mut model_hillclimb, output);
}
