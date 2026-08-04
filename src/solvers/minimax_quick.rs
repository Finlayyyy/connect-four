use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::BitBoard;
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};

/// A solver that extends `MinimaxABCached` and relies on `BitBoard`
/// being able to quickly calculate possible non-losing next moves.
/// Credit to
/// [Pascal Pons' Blog](http://blog.gamesolver.org/solving-connect-four/12-lower-bound-transposition-table/)
pub struct MinimaxQuick { }

impl ABSolver<BitBoard> for MinimaxQuick {
    fn minimax<M: SolverManager>(
        pos: BitBoard,
        boss: &mut M,
        cache: &mut Cache<BitBoard>,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        debug_assert!(!pos.curr_can_win());

        boss.check()?;
        if pos.is_full() { return ControlFlow::Continue(0); }
        let prev_alpha = alpha;

        let Ok(nexts) = pos.possible_nonlosing_nexts() else {
            return ControlFlow::Continue(pos.will_lose_eval());
        };

        // With at most 2 moves remaining and no win for curr
        // or opp, it must be a draw
        if pos.remaining_moves() <= 2 { return ControlFlow::Continue(0); }

        alpha = max(alpha, pos.will_lose_eval() + 1);
        beta = min(beta, pos.will_win_eval() - 1);
        if alpha >= beta {return ControlFlow::Continue(beta); }

        (alpha, beta) = cache.get_and_bound(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        let moves: MoveSorter<{ column::COUNT }, _, _> = nexts
            .map(|(_col, next_board)| (-next_board.heuristic(), next_board))
            .collect();


        for next_pos in moves {
            let eval = -Self::minimax(next_pos, boss, cache, -beta, -alpha)?;
            alpha = max(alpha, eval);
            if alpha >= beta { break; }
        }

        cache.insert_bounded(&pos, prev_alpha, alpha, beta);
        ControlFlow::Continue(alpha)
    }
}

impl Solver<BitBoard> for MinimaxQuick {
    fn solve<M: SolverManager>(pos: BitBoard, boss: &mut M, cache: &mut Cache<BitBoard>) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_eval();
        let max = pos.will_win_eval();
        if pos.curr_can_win() {
            return ControlFlow::Continue(pos.will_win_eval());
        } else {
            Self::minimax(pos, boss, cache, min, max)
        }

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxQuick | BitBoard);
}
