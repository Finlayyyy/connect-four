use std::ops::ControlFlow;

use crate::basic::{Cell, Token, column, row};
use crate::benching::{END_EASY, read_testset};
use crate::board::{Board, CloneBoard, MutBoard};
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};
use paste;

pub use crate::board::*;

macro_rules! make_solver_tests {
    ($solver:ty | $($pos:ty),+) => {
        $(
            paste::paste! {
                #[test]
                fn [< $pos:snake >]() {
                    crate::solvers::testing::run_easy_tests::<$pos, $solver>(1000);
                }
            }
        )+
    };
    ($count:expr => $solver:ty | $($pos:ty),+) => {
        $(
            paste::paste! {
                #[test]
                fn [< $pos:snake >]() {
                    crate::solvers::testing::run_easy_tests::<$pos, $solver>($count);
                }
            }
        )+
    };
}

pub fn run_easy_tests<P: Position, S: Solver<P>>(count: usize) {
    let mut boss = LaissezFaire {};
    let mut cache = Cache::new(Cache::SMALL_SIZE);

    for (moves, correct) in read_testset(END_EASY).into_iter().take(count) {
        let pos = P::from_moves(&moves);
        let ControlFlow::Continue(score) = S::solve(pos, &mut boss, &mut cache);
        assert_eq!(score, correct, "Solver failed moveset {moves}");
    }
}
