#![feature(generic_const_exprs, never_type)]
#![allow(incomplete_features, dead_code, unused_macros)]

mod basic;
#[macro_use]
mod bench;
mod board;
mod player;
mod solver_utils;
mod solvers;

use std::env;
use std::time::Duration;

use crate::bench::*;
use crate::board::*;
use crate::player::Player;
use crate::solver_utils::*;
use crate::solvers::*;

fn play(starting: bool) {
    println!();
    println!("You are playing against `Deepening<MinimaxQuick>` on `BitBoard`");
    if starting {
        println!("It will take the first move");
    } else {
        println!("You will take the first move");
    }
    println!("Good luck!");
    println!();

    let mut pos = BitBoard::EMPTY;
    let human = player::Human::new();
    let ai = player::Minimax::<BitBoard, Deepening<MinimaxQuick>>::new();

    let (mut curr, mut opp): (Box<dyn Player<BitBoard>>, Box<dyn Player<BitBoard>>) = if starting {
        (Box::new(ai), Box::new(human))
    } else {
        (Box::new(human), Box::new(ai))
    };

    while !pos.is_full() {
        println!("It's {}'s turn.", curr.name());
        let col = curr.play(&pos);
        pos.force_place(col, pos.curr());
        println!("{} played {}", curr.name(), col);
        println!();
        if pos.is_won_at_col(col) {
            println!();
            println!("{} has won! Here is the final position:", curr.name());
            pos.display();
            return;
        }
        (curr, opp) = (opp, curr);
    }
    println!();
    println!("The game is a draw! Here is the final position:");
    pos.display();
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
        100 from (END_EASY, MIDDLE_EASY, MIDDLE_MEDIUM) with Duration::from_secs(15)
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
    {
        let bencher = bencher!(
            (END_EASY, MIDDLE_EASY, MIDDLE_MEDIUM) with Duration::from_secs(30));
        bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
        println!();
    }
    {
        let bencher = bencher!(
            (BEGIN_EASY, BEGIN_MEDIUM, SOLVE) with Duration::from_mins(5));
        bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
        println!();
    }
    {
        let bencher = bencher!(
            100 from (BEGIN_HARD) with Duration::from_mins(9));
        bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
    }
    println!("Preliminary test complete.");
    {
        let bencher = bencher!(
            (BEGIN_HARD) with Duration::from_mins(90));
        bench!(bencher, BitBoard, Deepening<MinimaxQuick>);
    }
}

fn bench() {
    bench_very_easy();
    bench_easy();
    bench_medium();
    bench_hard();
}

fn display_usage() {
    println!("usage:                                      \n");
    println!("  ConnectFour [-h | --help]                 \n");
    println!("  ConnectFour bench                         \n");
    println!("  ConnectFour bench [easy|medium|hard|best]\n");
    println!("  ConnectFour help                          \n");
    println!("  ConnectFour play                          \n");
    println!("  ConnectFour play [first|second]           \n");
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
            let Some(testset) = args.next() else {
                return bench();
            };
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
        }
        "help" => display_usage(),
        "play" => {
            let Some(place) = args.next() else {
                return play(false);
            };
            match place.as_str() {
                "first" | "start" | "starting" => play(true),
                "second" => play(false),
                cmd => {
                    println!("Unrecognised command '{}'\n", cmd);
                    display_usage();
                }
            }
        }
        cmd => {
            println!("Unrecognised command '{}'\n", cmd);
            display_usage();
        }
    }
}
