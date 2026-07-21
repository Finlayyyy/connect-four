use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, thread, time::Duration};
use std::ops::ControlFlow;


pub trait SolverManager {
    type Break;

    fn check(&mut self) -> ControlFlow<Self::Break, ()>;
    fn log_bytes(&mut self, size: usize);
}

#[derive(Debug)]
pub struct LaissezFaire {}
impl SolverManager for LaissezFaire {
    type Break = !;
    fn check(&mut self) -> ControlFlow<!, ()> { ControlFlow::Continue(()) }
    fn log_bytes(&mut self, size: usize) { } // that's nice, don't care
}

#[derive(Debug)]
pub struct Logger {
    count: usize,
    alloc_size: usize
}
impl Logger {
    pub fn new() -> Self {
        Logger { count: 0, alloc_size: 0 }
    }

    pub fn count(&self) -> usize {
        self.count
    }
    pub fn alloc_size(&self) -> usize {
        self.alloc_size
    }
}
impl SolverManager for Logger {
    type Break = !;
    fn check(&mut self) -> ControlFlow<!, ()> { 
        self.count += 1;
        ControlFlow::Continue(()) 
    }

    fn log_bytes(&mut self, size: usize) { 
        self.alloc_size += size;
    } 
}


#[derive(Debug)]
pub struct WithTimeout<S> {
    timeout: Arc<AtomicBool>,
    pub inner: S
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
            true => ControlFlow::Break(())
        }
    }

    fn log_bytes(&mut self, size: usize) {
        self.inner.log_bytes(size);
    }
}


#[derive(Debug)]
pub struct Timeout<S> {
    max_time: Duration,
    pub timer: WithTimeout<S>
}

impl<S> Timeout<S> {
    pub fn new(max_time: Duration, inner: S) -> Self {
        let timeout = Arc::new(AtomicBool::new(false));
        Timeout { max_time, timer: WithTimeout::new(timeout, inner) }
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

    fn check(&mut self) -> ControlFlow<(), ()> { self.timer.check() }
    fn log_bytes(&mut self, size: usize) { self.timer.log_bytes(size); }
}

