#![feature(step_trait, generic_const_exprs, never_type)]
#![allow(dead_code, unused, incomplete_features)]

use crate::basic::*;
use crate::benching::*;
use crate::board::*;
use crate::solver_utils::*;
use crate::solvers::minimax_avoidant::minimax_avoidant_helper;
use crate::solvers::minimax_ordered::minimax_ordered_helper;
use crate::solvers::minimax_quick_avoid::minimax_quick_avoid_helper;
use crate::solvers::*;

use std::env;
use std::ops::ControlFlow;
use std::time::{Duration, Instant};

mod basic;
#[macro_use]
mod benching;
mod board;
mod solver_utils;
mod solvers;

fn play() {
    let moves = "444443433365666233222755555226617771";
    let moves = Moves::from_string(moves);
    let pos = BitBoard::from_moves(&moves);
    pos.display();

    let mut lower = HashMap::new(hash_map::LARGE_SIZE);
    let mut upper = HashMap::new(hash_map::LARGE_SIZE);
    let mut alpha = pos.will_lose_score();
    let mut beta = pos.will_win_score();

    let mut best_col = None;
    let mut best = isize::MIN;
    let mut broken = None;

    for (col, next) in pos.nexts(pos.curr()) {
        let mut boss = Timeout::new(Duration::from_secs(5), LaissezFaire {});
        boss.start_timer();
        let result =
            minimax_avoidant_helper(next, &mut boss, -beta, -alpha, &mut lower, &mut upper);
        println!("[{col}] => {result:?}");
        match result {
            ControlFlow::Continue(score) if -score > best => {
                best = -score;
                best_col = Some(col);
            }
            ControlFlow::Break(_) if broken.is_none() => {
                broken = Some(col);
            }
            _ => (),
        }
    }
    let col = match best_col {
        Some(col) if best < 0 && broken.is_some() => broken.unwrap(),
        Some(col) => col,
        None => broken.unwrap(),
    };
    println!();
    println!("Chose col `{col:?} with score `{best}");
    let next = pos.placed(col, pos.curr()).unwrap();
    next.display();
}

fn bench_very_easy() {
    println!("BENCH VERY EASY");
    let testsets = [END_EASY, MIDDLE_EASY];
    let tests = Bencher::read_testsets(&testsets, 100);
    let bencher = Bencher::new(tests, &testsets, Duration::from_secs(5));
    bench!(bencher, WithInfo<ArrayBoard>, minimax_mut);
    bench!(bencher, BitCols, minimax_mut);
    bench!(bencher, BitCols, minimax_clone);
    bench!(bencher, SymmBoard, minimax_mut);
    bench!(bencher, SymmBoard, minimax_clone);
}

fn bench_easy() {
    println!("BENCH EASY");
    let testsets = [END_EASY, MIDDLE_EASY, MIDDLE_MEDIUM];
    let tests = Bencher::read_testsets(&testsets, 100);
    let bencher = Bencher::new(tests, &testsets, Duration::from_secs(15));
    bench!(bencher, BitCols, minimax_alphabeta);
    bench!(bencher, WithInfo<BitCols>, minimax_alphabeta);
    println!();
    bench!(bencher, BitCols, minimax_cached);
    bench!(bencher, WithInfo<BitCols>, minimax_cached);
    bench!(bencher, SymmBoard, minimax_cached);
    bench!(bencher, WithInfo<SymmBoard>, minimax_cached);
    println!();
    bench!(bencher, BitCols, minimax_ab_cached);
    bench!(bencher, WithInfo<BitCols>, minimax_ab_cached);
    bench!(bencher, SymmBoard, minimax_ab_cached);
    bench!(bencher, WithInfo<SymmBoard>, minimax_ab_cached);
    bench!(bencher, BitBoard, minimax_ab_cached);
    println!();
    bench!(bencher, BitCols, minimax_symm);
    bench!(bencher, WithInfo<BitCols>, minimax_symm);
    bench!(bencher, SymmBoard, minimax_symm);
    bench!(bencher, WithInfo<SymmBoard>, minimax_symm);
    bench!(bencher, BitBoard, minimax_symm);
}

fn bench_medium() {
    println!("BENCH MEDIUM");
    let testsets = [MIDDLE_MEDIUM, BEGIN_EASY];
    let tests = Bencher::read_testsets(&testsets, 100);
    let bencher = Bencher::new(tests, &testsets, Duration::from_secs(30));
    bench!(bencher, BitCols, minimax_ordered);
    bench!(bencher, SymmBoard, minimax_ordered);
    println!();
    bench!(bencher, BitCols, minimax_avoidant);
    bench!(bencher, SymmBoard, minimax_avoidant);
    bench!(bencher, BitBoard, minimax_avoidant);
    println!();
    bench!(bencher, BitBoard, minimax_quick_avoid);
}

fn bench_hard() {
    println!("BENCH HARD");
    let testsets = [MIDDLE_MEDIUM, BEGIN_EASY, BEGIN_MEDIUM, BEGIN_HARD];
    let tests = Bencher::read_testsets(&testsets, 100);
    let bencher = Bencher::new(tests, &testsets, Duration::from_secs(60));
    bench!(
        bencher,
        BitCols,
        "deep_ordered<BitCols>",
        &minimax_deepening(&minimax_ordered_helper)
    );
    bench!(
        bencher,
        BitCols,
        "deep_avoidant<BitCols>",
        &minimax_deepening(&minimax_avoidant_helper)
    );
    bench!(
        bencher,
        SymmBoard,
        "deep_avoidant<SymmBoard>",
        &minimax_deepening(&minimax_avoidant_helper)
    );
    bench!(
        bencher,
        BitBoard,
        "deep_quick<BitBoard>",
        &minimax_deepening(&minimax_quick_avoid_helper)
    );
}

fn bench_solve() {
    println!("BENCH SOLVE");
    let tests = vec![vec![(Moves::EMPTY, 1)]];
    let bencher = Bencher::new(tests, &["SOLVE"], Duration::from_mins(30));
    bench!(
        bencher,
        BitBoard,
        "deep_quick<BitBoard>",
        &minimax_deepening(&minimax_quick_avoid_helper)
    );
}

fn bench() {
    bench_very_easy();
    bench_easy();
    bench_medium();
    bench_hard();
    bench_solve();
}

fn display_usage() {
    println!("usage:               \n");
    println!("       ConnectFour [-h | --help] \n");
    println!("       ConnectFour bench       \n");
    println!("       ConnectFour help        \n");
    println!("       ConnectFour play        \n");
}

fn main() {
    let mut args = env::args().skip(1);
    match args.next().map(|s| s.to_lowercase()).as_deref() {
        None => display_usage(),
        Some("--help") => display_usage(),
        Some("-h") => display_usage(),
        Some("bench") => match args.next().map(|s| s.to_lowercase()).as_deref() {
            None => bench(),
            Some("easy") => bench_very_easy(),
            Some("medium") => bench_easy(),
            Some("hard") => bench_medium(),
            Some("extreme") => bench_hard(),
            Some(cmd) => {
                println!("Unrecognised command '{}'\n", cmd);
                display_usage();
            }
        },
        Some("help") => display_usage(),
        Some("play") => play(),
        Some(cmd) => {
            println!("Unrecognised command '{}'\n", cmd);
            display_usage();
        }
    }
}
