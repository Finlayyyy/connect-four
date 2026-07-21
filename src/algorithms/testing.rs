
use crate::algorithms::{LaissezFaire, Position, SolverManager};
use crate::basic::{Cell, Token, column, row};
use crate::benching::{END_EASY, read_testset};
use crate::board::{Board, CloneBoard, MutBoard};


use paste;


macro_rules! make_test_with_board_on_position {
    ($func:expr, $b:ty) => {
        paste::paste! {
            #[test]
            fn [< $b:snake >]() {
                crate::algorithms::testing::run_easy_tests::<($b)>($func);
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

pub fn run_easy_tests<P: Position>(f: impl Fn(P) -> isize) {
    const COUNT: usize = 300;

    for (moves, score) in read_testset(END_EASY).into_iter().take(COUNT) {
        let pos = P::from_moves(&moves);
        let mut boss = LaissezFaire { };
        assert_eq!(f(pos), score, "Solver failed moveset {moves}");
    }
}
