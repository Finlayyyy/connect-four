use std::hash::Hash;
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::*;

#[macro_use]
mod testing;

pub mod minimax_ab_cached;
pub mod minimax_alphabeta;
pub mod minimax_avoidant;
pub mod minimax_basic;
pub mod minimax_cached;
pub mod minimax_deepening;
pub mod minimax_ordered;
pub mod minimax_quick;
pub mod minimax_symm;

pub use minimax_ab_cached::MinimaxABCached;
pub use minimax_alphabeta::MinimaxAlphaBeta;
pub use minimax_avoidant::MinimaxAvoidant;
pub use minimax_basic::MinimaxClone;
pub use minimax_basic::MinimaxMut;
pub use minimax_cached::MinimaxCached;
pub use minimax_deepening::Deepening;
pub use minimax_ordered::MinimaxOrdered;
pub use minimax_quick::MinimaxQuick;
pub use minimax_symm::MinimaxSymm;

use crate::solver_utils::*;

pub trait Solver<P> {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, cache: &mut Cache) -> ControlFlow<M::Break, isize>;

    fn just_solve(pos: P) -> isize {
        let mut boss = LaissezFaire { };
        let mut cache = Cache::new(Cache::SMALL_SIZE);
        match Self::solve(pos, &mut boss, &mut cache) {
            ControlFlow::Continue(score) => score
        }
    }
}

pub trait ABSolver<P>: Solver<P> {
    fn minimax<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache,
        alpha: isize,
        beta: isize,
    ) -> ControlFlow<M::Break, isize>;
}
