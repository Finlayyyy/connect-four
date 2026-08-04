use std::ops::ControlFlow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

/// A trait for managing the execution of a
/// solver. It provides a `check` method that
/// returns a `ControlFlow` indicating whether
/// the solver should continue or break.
pub trait SolverManager: Clone {
    type Break;

    fn check(&mut self) -> ControlFlow<Self::Break, ()>;
    fn count(&self) -> usize {
        0
    }
}


/// Allows the solver to continue indefinitely.
#[derive(Debug, Clone)]
pub struct LaissezFaire {}

impl SolverManager for LaissezFaire {
    type Break = !;

    fn check(&mut self) -> ControlFlow<!, ()> {
        ControlFlow::Continue(())
    }
}

/// Logs the number of iterations of the solver.
#[derive(Debug, Clone)]
pub struct Logger {
    count: usize,
}

impl Logger {
    pub fn new() -> Self {
        Logger { count: 0 }
    }
}

impl SolverManager for Logger {
    type Break = !;
    fn check(&mut self) -> ControlFlow<!, ()> {
        self.count += 1;
        ControlFlow::Continue(())
    }
    fn count(&self) -> usize {
        self.count
    }
}


/// Wraps an inner `SolverManager` with a timeout for
/// a given `Duration`. Starts a separate thread to
/// periodically check the timeout. The timeout begins
/// on creation
#[derive(Debug)]
pub struct Timeout<M> {
    max_time: Duration,
    timeout: Arc<AtomicBool>,
    pub inner: M
}

impl<M> Timeout<M> {
    pub fn new(max_time: Duration, inner: M) -> Self {
        let timeout = Arc::new(AtomicBool::new(false));
        let mut timeout = Timeout {
            max_time,
            timeout,
            inner,
        };
        timeout.start_timer();
        timeout
    }

    /// Start the timer
    fn start_timer(&mut self) {
        let max_time = self.max_time;
        let timeout = self.timeout.clone();
        thread::spawn(move || {
            thread::sleep(max_time);
            timeout.store(true, Ordering::Relaxed);
        });
    }
}

impl<M: SolverManager<Break = !>> SolverManager for Timeout<M> {
    type Break = ();

    fn check(&mut self) -> ControlFlow<(), ()> {
        let ControlFlow::Continue(()) = self.inner.check();
        if self.timeout.load(Ordering::Relaxed) {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn count(&self) -> usize {
        self.inner.count()
    }
}


impl<M: Clone> Clone for Timeout<M> {
    fn clone(&self) -> Self {
        Timeout::new(self.max_time, self.inner.clone())
    }
}
