use rand::seq::IndexedRandom;
use std::{fs, ops::ControlFlow, sync::{Arc, atomic::{AtomicBool, Ordering}, mpsc}, thread, time::{Duration, Instant}};


use crate::solver_utils::*;
use crate::basic::*;
use crate::board::*;

pub const END_EASY: &str = "Test_L3_R1";
pub const MIDDLE_EASY: &str = "Test_L2_R1";
pub const MIDDLE_MEDIUM: &str = "Test_L2_R2";
pub const BEGIN_EASY: &str = "Test_L1_R1";
pub const BEGIN_MEDIUM: &str = "Test_L1_R2";
pub const BEGIN_HARD: &str = "Test_L1_R3";

macro_rules! bench {
    ($bencher:ident, $B:ty, $solver:expr) => {
        $bencher.bench::<$B, _>(
            &format!("{}<{}>", stringify!($solver), stringify!($B)),
            &$solver);
    };

    ($bencher:ident, $B:ty, $name:expr, $solver:expr) => {
        $bencher.bench::<$B, _>(
            $name,
            &$solver);
    };

}

pub fn read_testset(string: &str) -> Vec<(Moves, isize)> {
    let contents = fs::read_to_string(format!("src/benching/positions/{}", string))
        .expect(format!("Could not read file: {}", string).as_str());

    let mut positions = Vec::new();
    for line in contents.lines() {
        let test = line.split_whitespace().collect::<Vec<&str>>();
        debug_assert!(test.len() == 2);
        
        let moves = test[0];
        let score = test[1];
        
        debug_assert!(moves.chars().all(|c| c.is_digit(10)));
        let moves = Moves::from_string(moves);

        match score.parse::<isize>() {
            Err(_) => {
                panic!("Invalid token in moves file: {}", score);
            }
            Ok(n) => positions.push((moves.to_owned(), n))
        }
    }

    positions
}

pub struct Bencher {
    tests: Vec<Vec<(Moves, isize)>>,
    names: Vec<String>,
    max_time: Duration,
}

impl Bencher {
    pub fn read_testsets(testsets: &[&str], count: usize) -> Vec<Vec<(Moves, isize)>> {
        testsets.iter()
            .map(|name| read_testset(name).into_iter().take(count).collect())
            .collect()
    }

    pub fn new(tests: Vec<Vec<(Moves, isize)>>, names: &[&str], max_time: Duration) -> Self {
        assert_eq!(tests.len(), names.len());
        print!("~~~ BENCH START ~~~ {:>10.0}ms         |", max_time.as_millis());
        for &name in names  {
            print!(" {:<19}|", name);
        }
        println!();
        print!("                                         |");
        for test in tests.iter() {
            print!("              /{:>4} |", test.len());
        }
        println!("");
        Bencher {
            tests,
            names: names.iter().map(|&s| s.to_owned()).collect(),
            max_time,
        }
    }

    pub fn bench<P, F>(&self, name: &str, f: & F)
    where 
        P: Position + Send + 'static,
        F: Fn(P, &mut Timeout<Logger>) -> ControlFlow<(), isize>,
        F: Sync,
        
    {
        print!("{:<40} |", name);
        
        for set in self.tests.iter() {
            let mut boss = Timeout::new(self.max_time, Logger::new());
            boss.start_timer();

            let durs = set
                .iter()
                .map(|(moves, score)| 
                    bench_func_on(moves, *score, f, &mut boss)
                )
                .map_while(|dur| dur);

            let (dur_len, dur_sum) = durs.fold((0,0), |(l, s), dur| (l + 1, s + dur));
            let mean_dur = dur_sum as f64 / dur_len as f64;
            let mean_count = boss.timer.inner.count() as f64 / dur_len as f64;

            print!("{:4.0}ms {:.2e}# {:4.0}/|", mean_dur, mean_count, dur_len);
            if dur_len < set.len() { break };
        }
        println!();
    }
}

impl Drop for Bencher {
    fn drop(&mut self) {
        print!("                               |");
        for _ in &self.tests  {
           print!("-------------|");
        }
        println!("");
    }
}

fn bench_func_on<P, S, F>(moves: &Moves, correct: isize, f: &F, boss: &mut S) -> Option<usize>
where 
    P: Position,
    S: SolverManager,
    F: Fn(P, &mut S) -> ControlFlow<S::Break, isize>,
{
    let pos = P::from_moves(moves);
    
    let start = Instant::now();
    let ControlFlow::Continue(score) = f(pos, boss) else {
        return None;
    };
    let dur = start.elapsed().as_millis();

    assert_eq!(score, correct, "Score assertion failed for moves {}", moves);
    return Some(dur as usize)
}