use crate::core::{PurrHillClimbModel, PurrModel, PurrShape, PurrState};
use crossbeam_channel::{Receiver, Sender};

pub enum PurrWorkerCmd<T: PurrShape> {
    Start,
    Step(PurrState<T>),
    End,
}

pub struct PurrWorker<T: PurrShape> {
    model: PurrHillClimbModel,
    rx: Receiver<PurrWorkerCmd<T>>,
    tx: Sender<PurrState<T>>,
}

impl<T: PurrShape> PurrWorker<T> {
    pub fn new(
        model: PurrHillClimbModel,
        rx: Receiver<PurrWorkerCmd<T>>,
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
                PurrWorkerCmd::Step(_state) => {
                    self.update(_state);
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

    pub fn update(&mut self, state: PurrState<T>) {
        self.model.add_state(&state);
    }
}
