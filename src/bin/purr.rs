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
                .long("input")
                .help("Input Image")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Output Image")
                .required(true)
                .takes_value(true),
        )
        .get_matches();
    let input = matches.value_of("input").unwrap();
    let output = matches.value_of("output").unwrap();
    let mut runner: PurrModelRunner<Triangle> = PurrModelRunner::default();
    let model_ctx = PurrContext::new(input);
    let mut model = PurrModel::new(model_ctx, 1000, 4, 100);
    runner.run(&mut model, output);
}
