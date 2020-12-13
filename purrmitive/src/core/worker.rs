use crate::core::{PurrHillClimbModel, PurrModel, PurrShape, PurrState};
use crossbeam_channel::{Receiver, Sender};

pub enum PurrWorkerCmd {
    Start,
    UpdateScore(f64),
    End,
}

pub struct PurrWorker<T: PurrShape> {
    model: PurrHillClimbModel,
    rx: Receiver<PurrWorkerCmd>,
    tx: Sender<PurrState<T>>,
}

impl<T: PurrShape> PurrWorker<T> {
    pub fn new(
        model: PurrHillClimbModel,
        rx: Receiver<PurrWorkerCmd>,
        tx: Sender<PurrState<T>>,
    ) -> Self {
        PurrWorker { model, rx, tx }
    }

    pub fn start(&mut self) {
        loop {
            let cmd = self.rx.recv().unwrap();
            match cmd {
                PurrWorkerCmd::Start => {
                    self.work();
                }
                PurrWorkerCmd::UpdateScore(s) => {
                    self.model.context.score = s;
                }
                PurrWorkerCmd::End => {
                    return;
                }
            }
        }
    }

    pub fn work(&mut self) {
        let state = self.model.step();
        self.tx.send(state).unwrap();
    }
}
