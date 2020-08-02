use purr::core::*;
use purr::graphics::*;

fn main() {
    let mut runner: PurrModelRunner<Triangle> = PurrModelRunner::default();
    let model_ctx = PurrContext::new("input.png");
    let mut model = PurrModel::new(model_ctx, 1000, 4, 100);
    runner.run(&mut model);
}
