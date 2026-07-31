use std::ops::ControlFlow;

use crate::bench::END_EASY;
use crate::solver_utils::*;
use crate::solvers::Solver;

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

/// Run a series of tests on the `END_EASY` testset using the given solver.
pub fn run_easy_tests<P: Position, S: Solver<P>>(count: usize) {
    let mut boss = LaissezFaire {};
    let mut cache = Cache::new_small();

    for (moves, correct) in END_EASY.iter().take(count) {
        let pos = P::from_moves(moves);
        let ControlFlow::Continue(eval) = S::solve(pos, &mut boss, &mut cache);
        assert_eq!(eval, *correct, "Solver failed moveset {moves}");
    }
}
