use purr::core::*;

fn main() {
    let mut runner = PurrModelRunner::default();
    let model_ctx = PurrContext::new("input.png");
    let mut model = PurrModel::new(model_ctx);
    runner.run(&mut model);
}
