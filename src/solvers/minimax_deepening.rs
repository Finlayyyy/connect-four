use std::ops::ControlFlow;
use std::marker::PhantomData;

use crate::board::{CloneBoard, HashBoard};
use crate::solver_utils::*;
use crate::solvers::{ABSolver, Solver};

/// A solver that wraps an alpha-beta solver that uses
/// iterative deepening
pub struct Deepening<S> {
    pd: PhantomData<S>,
}

impl<P, S> Solver<P> for Deepening<S>
where
    P: Position + CloneBoard + HashBoard,
    S: ABSolver<P>,
{
    fn solve<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache<P>,
    ) -> ControlFlow<M::Break, isize> {
        if pos.can_win(pos.curr()) {
            return ControlFlow::Continue(pos.will_win_score());
        }

        boss.check()?;
        let mut min = pos.will_lose_score();
        let mut max = pos.will_win_score() - 1;

        while min < max {
            let mut mid = min + (max - min) / 2;
            if mid <= 0 && min / 2 < mid {
                mid = min / 2
            };
            if mid >= 0 && max / 2 > mid {
                mid = max / 2
            };
            let score = S::minimax(pos.clone(), boss, cache, mid, mid + 1)?;
            if score <= mid {
                max = score
            } else {
                min = score
            };
        }
        ControlFlow::Continue(min)
    }
}

#[cfg(test)]
mod ordered_tests {
    use super::*;
    use crate::solvers::MinimaxOrdered;
    use crate::solvers::testing::*;

    make_solver_tests!(
        Deepening<MinimaxOrdered> |
        BitCols,
        BitBoard,
        SymmBoard
    );
}

#[cfg(test)]
mod avoidant_tests {
    use super::*;
    use crate::solvers::MinimaxAvoidant;
    use crate::solvers::testing::*;

    make_solver_tests!(
        Deepening<MinimaxAvoidant> |
        BitCols,
        BitBoard,
        SymmBoard
    );
}
