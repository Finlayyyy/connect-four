#![feature(generic_const_exprs, never_type)]
#![allow(incomplete_features, dead_code, unused_macros)]

use crate::benching::*;
use crate::board::*;
use crate::solver_utils::*;
use crate::solvers::*;

use std::env;
use std::time::Duration;

mod basic;
#[macro_use]
mod benching;
mod board;
mod solver_utils;
mod solvers;

fn play() {
    todo!();
}

fn bench_very_easy() {
    println!("BENCH VERY EASY");
    let bencher = bencher!(
        100 from (END_EASY, MIDDLE_EASY) with Duration::from_secs(1)
    );
    bench!(bencher, WithInfo<ArrayBoard>, MinimaxMut);
    bench!(bencher, BitCols, MinimaxMut);
    bench!(bencher, BitCols, MinimaxClone);
    bench!(bencher, SymmBoard, MinimaxMut);
    bench!(bencher, SymmBoard, MinimaxClone);
}

fn bench_easy() {
    println!("BENCH EASY");
    let bencher = bencher!(
        100 from (END_EASY, MIDDLE_EASY, MIDDLE_MEDIUM) with Duration::from_secs(5)
    );

    bench!(bencher, BitCols, MinimaxAlphaBeta);
    bench!(bencher, WithInfo<BitCols>, MinimaxAlphaBeta);
    println!();
    bench!(bencher, BitCols, MinimaxCached);
    bench!(bencher, WithInfo<BitCols>, MinimaxCached);
    bench!(bencher, SymmBoard, MinimaxCached);
    bench!(bencher, WithInfo<SymmBoard>, MinimaxCached);
    println!();
    bench!(bencher, BitCols, MinimaxABCached);
    bench!(bencher, WithInfo<BitCols>, MinimaxABCached);
    bench!(bencher, SymmBoard, MinimaxABCached);
    bench!(bencher, WithInfo<SymmBoard>, MinimaxABCached);
    bench!(bencher, BitBoard, MinimaxABCached);
    println!();
    bench!(bencher, BitCols, MinimaxSymm);
    bench!(bencher, WithInfo<BitCols>, MinimaxSymm);
    bench!(bencher, SymmBoard, MinimaxSymm);
    bench!(bencher, WithInfo<SymmBoard>, MinimaxSymm);
    bench!(bencher, BitBoard, MinimaxSymm);
}

fn bench_medium() {
    println!("BENCH MEDIUM");
    let bencher = bencher!(
        100 from (MIDDLE_EASY, MIDDLE_MEDIUM, BEGIN_EASY) with Duration::from_secs(15)
    );
    bench!(bencher, BitCols, MinimaxOrdered);
    bench!(bencher, SymmBoard, MinimaxOrdered);
    println!();
    bench!(bencher, BitCols, MinimaxAvoidant);
    bench!(bencher, SymmBoard, MinimaxAvoidant);
    bench!(bencher, BitBoard, MinimaxAvoidant);
    println!();
    bench!(bencher, BitBoard, MinimaxQuick);
}

fn bench_hard() {
    println!("BENCH HARD");
    let bencher = bencher!(
        100 from (MIDDLE_MEDIUM, BEGIN_EASY, BEGIN_MEDIUM, BEGIN_HARD)
        with Duration::from_secs(30)
    );

    bench!(bencher, BitCols, Deepening<MinimaxOrdered>);
    bench!(bencher, BitCols, Deepening<MinimaxAvoidant>);
    bench!(bencher, SymmBoard, Deepening<MinimaxAvoidant>);
    bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
}

fn bench_best() {
    println!("BENCH BEST");
    let bencher = bencher!((END_EASY, MIDDLE_EASY, MIDDLE_MEDIUM) with Duration::from_secs(30));
    bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
    println!();
    let bencher = bencher!((BEGIN_EASY, BEGIN_MEDIUM, SOLVE) with Duration::from_mins(5));
    bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
    println!();
    let bencher = bencher!((BEGIN_HARD) with Duration::from_mins(90));
    bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
}

fn bench() {
    bench_very_easy();
    bench_easy();
    bench_medium();
    bench_hard();
}

fn generate() {
    todo!()
}

fn display_usage() {
    println!("usage:                                      \n");
    println!("  ConnectFour [-h | --help]                 \n");
    println!("  ConnectFour bench                         \n");
    println!("  ConnectFour bench [easy|medium|hard|best]\n");
    println!("  ConnectFour generate                      \n");
    println!("  ConnectFour help                          \n");
    println!("  ConnectFour play                          \n");
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        println!("Expected command.\n");
        return display_usage();
    };
    let cmd = cmd.to_lowercase();
    match cmd.as_str() {
        "--help" => display_usage(),
        "-h" => display_usage(),
        "bench" => {
            let Some(testset) = args.next() else { return bench(); };
            match testset.as_str() {
                "easy" => bench_easy(),
                "medium" => bench_medium(),
                "hard" => bench_hard(),
                "best" => bench_best(),
                cmd => {
                    println!("Unrecognised command '{}'\n", cmd);
                    display_usage();
                }
            }
        },
        "generate" => generate(),
        "help" => display_usage(),
        "play" => play(),
        cmd => {
            println!("Unrecognised command '{}'\n", cmd);
            display_usage();
        }
    }


}
