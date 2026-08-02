use crate::board::CloneBoard;
use std::cmp::max;
use std::cmp::min;
use std::ops::ControlFlow;

use crate::solver_utils::*;
use crate::solvers::Solver;


/// Extends `MinimaxClone` with alpha-beta pruning.
pub struct MinimaxAlphaBeta {}
impl MinimaxAlphaBeta {
    fn minimax<P: Position + CloneBoard, M: SolverManager>(
        pos: &P,
        boss: &mut M,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.is_full() { return ControlFlow::Continue(0); }

        beta = min(beta, pos.will_win_eval());
        if alpha >= beta { return ControlFlow::Continue(beta); };

        for (col, next_pos) in pos.nexts(pos.curr()) {
            if next_pos.is_won_at_col(col) {
                return ControlFlow::Continue(next_pos.just_won_eval());
            }
            let eval = -Self::minimax(&next_pos, boss, -beta, -alpha)?;
            if eval >= beta { return ControlFlow::Continue(eval); }
            alpha = max(alpha, eval);
        }
        ControlFlow::Continue(alpha)
    }
}
impl<P: Position + CloneBoard> Solver<P> for MinimaxAlphaBeta {
    fn solve<M: SolverManager>(
        pos: P,
        boss: &mut M,
        _cache: &mut Cache<P>,
    ) -> ControlFlow<M::Break, isize> {
        Self::minimax(&pos, boss, pos.will_lose_eval(), pos.will_win_eval())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxAlphaBeta | BitCols, BitBoard, SymmBoard);
}
