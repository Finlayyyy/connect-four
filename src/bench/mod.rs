use std::fs;
use std::io;
use std::io::Write;
use std::ops::ControlFlow;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use crate::board::*;
use crate::solver_utils::*;
use crate::solvers::Solver;

/// A testset with a name and vec of tests,
/// a pair of Moves and true eval
#[derive(Debug, Clone)]
pub struct TestSet {
    name: String,
    tests: Vec<(Moves, isize)>,
}

impl TestSet {
    /// Loads tests from the given file name
    fn load_tests(set: &str) -> Vec<(Moves, isize)> {
        let contents = fs::read_to_string(format!("testsets/{}", set))
            .unwrap_or_else(|_| panic!("Could not read file: {}", set));

        let mut tests = Vec::new();
        for line in contents.lines() {
            let test = line.split_whitespace().collect::<Vec<&str>>();
            assert!(test.len() == 2);

            let moves = test[0];
            let eval = test[1];

            assert!(moves.chars().all(|c| c.is_ascii_digit()));
            let moves = Moves::from_string(moves);
            let s = eval
                .parse::<isize>()
                .unwrap_or_else(|_| panic!("Invalid token in moves file: {}", eval));
            tests.push((moves, s));
        }
        tests
    }

    /// Construct a new testset with the given name and tests
    pub fn new(name: &str, tests: Vec<(Moves, isize)>) -> Self {
        TestSet {
            name: name.to_owned(),
            tests
        }
    }

    /// Load a testset with the given name and file name
    pub fn load(name: &str, set: &str) -> Self {
        TestSet {
            name: name.to_owned(),
            tests: Self::load_tests(set),
        }
    }

    /// Name of the testset
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Number of tests in the testset
    pub fn len(&self) -> usize {
        self.tests.len()
    }

    /// Tests in the testset
    pub fn tests(&self) -> &[(Moves, isize)] {
        &self.tests
    }

    /// Take the first `count` tests from the testset
    pub fn take(self, count: usize) -> Self {
        TestSet {
            name: self.name,
            tests: self.tests.into_iter().take(count).collect() }
    }

    /// Iterate over the tests in the testset
    pub fn iter(&self) -> std::slice::Iter<'_, (Moves, isize)> {
        self.tests.iter()
    }
}

impl<'a> IntoIterator for &'a TestSet {
    type Item = &'a (Moves, isize);
    type IntoIter = std::slice::Iter<'a, (Moves, isize)>;

    fn into_iter(self) -> Self::IntoIter {
        self.tests.iter()
    }
}

/// Testset with `28 < move_count` and `remaining < 14`
pub static END_EASY: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("END_EASY", "Test_L3_R1"));
/// Testset with `14 < move_count <= 28` and `remaining < 14`
pub static MIDDLE_EASY: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("MIDDLE_EASY", "Test_L2_R1"));
/// Testset with `14 < move_count <= 28` and `14 <= remaining < 28`
pub static MIDDLE_MEDIUM: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("MIDDLE_MEDIUM", "Test_L2_R2"));
/// Testset with `move_count <= 14` and `remaining < 14`
pub static BEGIN_EASY: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("BEGIN_EASY", "Test_L1_R1"));
/// Testset with `move_count <= 14` and `14 <= remaining < 28`
pub static BEGIN_MEDIUM: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("BEGIN_MEDIUM", "Test_L1_R2"));
/// Testset with `move_count <= 14` and `28 <= remaining`
pub static BEGIN_HARD: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::load("BEGIN_HARD", "Test_L1_R3"));
/// Testset with `move_count == 0` and `remaining == 42`
pub static SOLVE: LazyLock<TestSet> =
    LazyLock::new(|| TestSet::new("SOLVE", vec![(Moves::EMPTY, 1)]));

/// Create a bencher with the given test sets.
macro_rules! bencher {
    (($($set:expr),+) with $dur:expr) => {
        Bencher::new(
            vec![$( $set.clone() ),+],
            $dur
        )
    };
    ($count:literal from ($($set:expr),+) with $dur:expr) => {
        Bencher::new(
            vec![ $( $set.clone().take($count) ),+],
            $dur
        )
    };
}

/// Use the bencher to bench the given combination
/// of board and solver
macro_rules! bench {
    ($bencher:ident, $B:ty, $solver:ty) => {
        $bencher.bench::<$B, $solver>(&format!("{}/{}", stringify!($solver), stringify!($B)));
    };
}

pub struct Bencher {
    cases: Vec<TestSet>,
    max_time: Duration,
}

impl Bencher {
    /// Create a new bencher with given testsets and max_duration per testset
    pub fn new(cases: Vec<TestSet>, max_time: Duration) -> Self {
        print!(
            "~~~ BENCH START ~~~ {:>10.0}s          |",
            max_time.as_secs()
        );
        for set in &cases {
            print!(" {:<19}|", set.name());
        }
        println!();
        print!("                                         |");
        for set in &cases {
            print!("              /{:>4} |", set.len());
        }
        println!();
        Bencher { cases, max_time }
    }

    /// Bench the given combination of Board and Solver
    pub fn bench<P, S>(&self, name: &str)
    where
        P: Position + Send + 'static,
        S: Solver<P>,
    {
        print!("{:<40} |", name);

        for testset in &self.cases {
            let mut boss = Timeout::new(self.max_time, Logger::new());
            boss.start_timer();

            let mut cache = Cache::new_large();

            let (dur_len, dur_sum) = testset
                .iter()
                .map_while(|(moves, eval)| {
                    bench_func_on::<S, _, _>(&mut boss, &mut cache, moves, *eval)
                })
                .fold((0, 0), |(l, s), dur| (l + 1, s + dur));

            let mean_dur = dur_sum as f64 / dur_len as f64;
            let mean_count = boss.inner.count() as f64 / dur_len as f64;

            print!("{:4.0}ms {:.2e}# {:4.0}/|", mean_dur, mean_count, dur_len);
            io::stdout().flush().unwrap();

            if dur_len < testset.len() {
                break;
            };
        }
        println!();
    }
}

impl Drop for Bencher {
    fn drop(&mut self) {
        print!("                                         |");
        for _ in &self.cases {
            print!("--------------------|");
        }
        println!();
    }
}

/// Runs the given solver on the given position with the
/// boss and cache. Returns `None` if the solver does not finish,
/// otherwise the number of milliseconds the solver ran for.
fn bench_func_on<S, P, M>(
    boss: &mut M,
    cache: &mut Cache<P>,
    moves: &Moves,
    correct: isize,
) -> Option<usize>
where
    P: Position,
    M: SolverManager,
    S: Solver<P>,
{
    let pos = P::from_moves(moves);

    let start = Instant::now();
    let ControlFlow::Continue(eval) = S::solve(pos, boss, cache) else {
        return None;
    };
    let dur = start.elapsed().as_millis();

    assert_eq!(eval, correct, "Eval assertion failed for moves {}", moves);
    Some(usize::try_from(dur).unwrap())
}
