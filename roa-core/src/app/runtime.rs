use super::FutureObj;
use crate::{App, Spawn, State};

impl<S: State> App<S> {
    /// Construct app with default runtime.
    pub fn new(state: S) -> Self {
        Self::with_exec(state, Executor)
    }
}

struct Executor;

impl Spawn for Executor {
    fn spawn(&self, fut: FutureObj) {
        async_std::task::spawn(fut);
    }

    fn spawn_blocking(&self, task: Box<dyn 'static + Send + FnOnce()>) {
        async_std::task::spawn_blocking(task);
    }
}