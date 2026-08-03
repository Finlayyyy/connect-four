use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard};
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};

/// Extends `MinimaxABCached` with move ordering.
/// Moves are sorted by the number of adjacent tokens
/// of the current player
pub struct MinimaxOrdered {}

impl<P: Position + CloneBoard + HashBoard> ABSolver<P> for MinimaxOrdered {
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

        (alpha, beta) = cache.get_and_bound(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        let mut moves = MoveSorter::<{ column::COUNT }, _, _>::new();

        for (col, next_pos) in pos.nexts(pos.curr()) {
            let cell = next_pos.top(col).unwrap();
            match next_pos.count_adjacent_around(cell, pos.curr()) {
                // found a win
                None => {
                    let eval = next_pos.just_won_eval();
                    return ControlFlow::Continue(eval);
                }
                // add the board
                Some(adjs) => moves.push_sorting(adjs, (col, next_pos)),
            }
        }

        alpha = max(alpha, pos.will_lose_eval());
        beta = min(beta, pos.will_win_eval() - 1);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        for (_col, next_pos) in moves {
            let eval = -Self::minimax(next_pos, boss, cache, -beta, -alpha)?;
            alpha = max(alpha, eval);
            if alpha >= beta { break; }
        }

        cache.insert_bounded(&pos, prev_alpha, alpha, beta);
        ControlFlow::Continue(alpha)
    }
}

impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxOrdered {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, cache: &mut Cache<P>) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_eval();
        let max = pos.will_win_eval();
        Self::minimax(pos, boss, cache, min, max)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxOrdered | BitCols, BitBoard, SymmBoard);
}
