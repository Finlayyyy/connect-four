use std::ops::ControlFlow;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub trait SolverManager {
    type Break;

    fn check(&mut self) -> ControlFlow<Self::Break, ()>;
}
#[derive(Debug)]
pub struct LaissezFaire {}
impl SolverManager for LaissezFaire {
    type Break = !;
    fn check(&mut self) -> ControlFlow<!, ()> {
        ControlFlow::Continue(())
    }
}

#[derive(Debug)]
pub struct Logger {
    count: usize,
}
impl Logger {
    pub fn new() -> Self {
        Logger { count: 0 }
    }

    pub fn count(&self) -> usize {
        self.count
    }
}
impl SolverManager for Logger {
    type Break = !;
    fn check(&mut self) -> ControlFlow<!, ()> {
        self.count += 1;
        ControlFlow::Continue(())
    }
}

#[derive(Debug)]
pub struct WithTimeout<S> {
    timeout: Arc<AtomicBool>,
    pub inner: S,
}

impl<S> WithTimeout<S> {
    pub fn new(timeout: Arc<AtomicBool>, inner: S) -> Self {
        WithTimeout { timeout, inner }
    }
}
impl<S: SolverManager<Break = !>> SolverManager for WithTimeout<S> {
    type Break = ();

    fn check(&mut self) -> ControlFlow<(), ()> {
        let ControlFlow::Continue(()) = self.inner.check();
        match self.timeout.load(Ordering::Relaxed) {
            false => ControlFlow::Continue(()),
            true => ControlFlow::Break(()),
        }
    }
}

#[derive(Debug)]
pub struct Timeout<S> {
    max_time: Duration,
    pub timer: WithTimeout<S>,
}

impl<S> Timeout<S> {
    pub fn new(max_time: Duration, inner: S) -> Self {
        let timeout = Arc::new(AtomicBool::new(false));
        Timeout {
            max_time,
            timer: WithTimeout::new(timeout, inner),
        }
    }

    pub fn start_timer(&mut self) {
        let max_time = self.max_time;
        let timeout = self.timer.timeout.clone();
        thread::spawn(move || {
            thread::sleep(max_time);
            timeout.store(true, Ordering::Relaxed);
        });
    }
}
impl<S: SolverManager<Break = !>> SolverManager for Timeout<S> {
    type Break = ();

    fn check(&mut self) -> ControlFlow<(), ()> {
        self.timer.check()
    }
}
