use std::ops::ControlFlow;

use crate::basic::{Cell, Token, column, row};
use crate::benching::{END_EASY, read_testset};
use crate::board::{Board, CloneBoard, MutBoard};
use crate::solver_utils::*;

pub use crate::board::*;

use paste;

macro_rules! make_test_with_board_on_position {
    ($func:expr, $b:ty) => {
        paste::paste! {
            #[test]
            fn [< $b:snake >]() {
                crate::solvers::testing::run_easy_tests::<($b)>($func);
            }
        }
    };

    ($func:expr, $name:ident, $b:ty) => {
        paste::paste! {
            #[test]
            fn $name() {
                crate::algorithms::testing::run_easy_tests::<($b)>($func);
            }
        }
    };
}

macro_rules! make_solver_tests {
    ($func:expr, $($b:ty),+) => {
        $(
            make_test_with_board_on_position!($func, $b);
        )+
    };
}

pub fn solve_using<P, F>(solver: &F) -> impl Fn(P) -> isize
where
    F: Fn(P, &mut LaissezFaire) -> ControlFlow<!, isize>,
{
    |pos| match solver(pos, &mut LaissezFaire {}) {
        ControlFlow::Continue(score) => score,
        ControlFlow::Break(_) => unreachable!(),
    }
}

pub fn run_easy_tests<P: Position>(f: impl Fn(P) -> isize) {
    const COUNT: usize = 300;

    for (moves, score) in read_testset(END_EASY).into_iter().take(COUNT) {
        let pos = P::from_moves(&moves);
        let mut boss = LaissezFaire {};
        assert_eq!(f(pos), score, "Solver failed moveset {moves}");
    }
}
