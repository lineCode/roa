use crate::executor::{BlockingObj, FutureObj};
use crate::{App, Spawn};

use tokio::task::{spawn, spawn_blocking};

impl<S> App<S, ()> {
    /// Construct app with default runtime.
    ///
    /// Feature `runtime` is required.
    #[inline]
    pub fn new(state: S) -> Self {
        Self::with_exec(state, Exec)
    }
}

pub struct Exec;

impl Spawn for Exec {
    #[inline]
    fn spawn(&self, fut: FutureObj) {
        spawn(fut);
    }

    #[inline]
    fn spawn_blocking(&self, task: BlockingObj) {
        spawn_blocking(task);
    }
}
