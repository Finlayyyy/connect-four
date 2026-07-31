use std::cmp::max;
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, MutBoard};
use crate::solver_utils::*;
use crate::solvers::Solver;

/// A basic minimax solver that uses a mutable board.
pub struct MinimaxMut { }
impl MinimaxMut {
    fn minimax<P: Position + MutBoard, M: SolverManager>(
        pos: &mut P,
        boss: &mut M
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;

        if pos.is_full() {
            return ControlFlow::Continue(0);
        }

        let mut best = isize::MIN;

        for col in column::CENTRED {
            let Some(cell) = pos.place_curr(col) else {
                continue;
            };
            if pos.is_won_at(cell) {
                let eval = pos.just_won_eval();
                pos.unplace(col);
                return ControlFlow::Continue(eval);
            }

            let eval = -Self::minimax(pos, boss)?;
            pos.unplace(col);
            best = max(best, eval);
        }

        ControlFlow::Continue(best)
    }
}

impl<P: Position + MutBoard> Solver<P> for MinimaxMut {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, _cache: &mut Cache<P>) -> ControlFlow<M::Break, isize> {
        let mut pos = pos;
        Self::minimax(&mut pos, boss)
    }
}

/// A basic minimax solver that uses a clone board.
pub struct MinimaxClone { }

impl MinimaxClone {
    fn minimax<P: Position + CloneBoard, M: SolverManager>(
        pos: P,
        boss: &mut M
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;

        if pos.is_full() {
            return ControlFlow::Continue(0);
        }

        let mut best = isize::MIN;

        for (col, next_pos) in pos.nexts(pos.curr()) {
            if next_pos.is_won_at_col(col) {
                return ControlFlow::Continue(next_pos.just_won_eval());
            }

            let eval = -Self::minimax(next_pos, boss)?;
            best = max(best, eval);
        }
        ControlFlow::Continue(best)
    }
}

impl<P: Position + CloneBoard> Solver<P> for MinimaxClone {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, _cache: &mut Cache<P>) -> ControlFlow<M::Break, isize> {
        Self::minimax(pos, boss)
    }
}

#[cfg(test)]
mod mut_tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(100 => MinimaxMut | BitCols);
}

#[cfg(test)]
mod clone_tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(100 => MinimaxClone | BitCols);
}
