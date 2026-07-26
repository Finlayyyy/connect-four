use crate::basic::*;
use crate::board::{CloneBoard, MutBoard};
use std::cmp::max;
use std::cmp::min;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::solver_utils::*;
use crate::solvers::Solver;

pub struct MinimaxAlphaBeta {}
impl MinimaxAlphaBeta {
    fn minimax<P: Position + CloneBoard, M: SolverManager>(
        pos: &P,
        boss: &mut M,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.completed() { return ControlFlow::Continue(0); }

        beta = min(beta, pos.will_win_score());
        if (alpha >= beta) { return ControlFlow::Continue(beta); };

        for (col, next_pos) in pos.nexts(pos.curr()) {
            if next_pos.is_won_at_col(col) {
                return ControlFlow::Continue(next_pos.just_won_score());
            }
            let score = -Self::minimax(&next_pos, boss, -beta, -alpha)?;
            if score >= beta { return ControlFlow::Continue(score); }
            alpha = max(alpha, score);
        }
        ControlFlow::Continue(alpha)
    }
}
impl<P: Position + CloneBoard> Solver<P> for MinimaxAlphaBeta {
    fn solve<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache,
    ) -> ControlFlow<M::Break, isize> {
        Self::minimax(&pos, boss, pos.will_lose_score(), pos.will_win_score())
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxAlphaBeta | BitCols, BitBoard, SymmBoard);
}
