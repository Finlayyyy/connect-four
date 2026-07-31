use std::cmp::{max, min};
use std::ops::ControlFlow;
use crate::board::{CloneBoard, HashBoard};

use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};


/// Combines `MinimaxAlphaBeta` and `MinimaxCaching`,
/// a solver that caches results as Lower, Exact, Upper
pub struct MinimaxABCached { }

impl<P: Position + CloneBoard + HashBoard> ABSolver<P> for MinimaxABCached  {
    fn minimax<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache<P>,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.is_full() { return ControlFlow::Continue(0); }

        let prev_alpha = alpha;
        beta = min(beta, pos.eval());
        (alpha, beta) = cache.get_and_bound(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        for (col, next_pos) in pos.nexts(pos.curr()) {
            if next_pos.is_won_at_col(col) {
                alpha = next_pos.just_won_eval();
                break;
            }

            let eval = -Self::minimax(next_pos, boss, cache, -beta, -alpha)?;

            alpha = max(alpha, eval);
            if alpha >= beta { break }
        }
        cache.insert_bounded(&pos, prev_alpha, alpha, beta);
        ControlFlow::Continue(alpha)
    }
}

impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxABCached {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, cache: &mut Cache<P>) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_eval();
        let max = pos.eval();
        Self::minimax(pos, boss, cache, min, max)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxABCached | BitCols, BitBoard, SymmBoard);
}
