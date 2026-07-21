use crate::basic::*;
use crate::board::{CloneBoard, MutBoard};
use std::cmp::max;
use std::cmp::min;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::solver_utils::*;

pub fn minimax_alphabeta<P: Position + CloneBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    minimax_alphabeta_helper(pos, boss, position::MIN_SCORE, position::MAX_SCORE)
}

fn minimax_alphabeta_helper<P: Position + CloneBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    mut alpha: isize,
    mut beta: isize,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;

    if pos.completed() {
        return ControlFlow::Continue(0);
    }

    beta = min(beta, pos.will_win_score());
    if (alpha >= beta) { return ControlFlow::Continue(beta) };

    for (col, next_pos) in pos.nexts(pos.curr()) {
        if next_pos.is_won_at_col(col) {
            return ControlFlow::Continue(next_pos.just_won_score());
        }
        let score = -minimax_alphabeta_helper(next_pos, boss, -beta, -alpha)?;
        if score >= beta { return ControlFlow::Continue(score) };
        alpha = max(alpha, score);
    }
    ControlFlow::Continue(alpha)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(solve_using(&minimax_alphabeta), BitCols, BitBoard, SymmBoard);

}
