use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{BitBoard, Board, CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};

pub struct MinimaxQuick { }

impl ABSolver<BitBoard> for MinimaxQuick {
    fn minimax<M: SolverManager>(
        pos: BitBoard,
        boss: &mut M,
        cache: &mut Cache,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.completed() { return ControlFlow::Continue(0); }
        debug_assert!(!pos.curr_can_win());

        let Ok(nexts) = pos.possible_nonlosing_nexts() else {
            return ControlFlow::Continue(pos.will_lose_score());
        };

        let moves: MoveSorter<{ column::COUNT }, _, _> = nexts
            .map(|(col, next_board)| (next_board.heuristic(), col))
            .collect();

        alpha = max(alpha, pos.will_lose_score() + 1);
        beta = min(beta, pos.will_win_score() - 1);
        (alpha, beta) = cache.check(&pos, alpha, beta);
        if alpha >= beta {return ControlFlow::Continue(beta); }

        let lower = alpha;
        let mut best = isize::MIN;
        for col in moves {
            let next_pos = pos.placed_curr_unchecked(col);
            let score = -Self::minimax(next_pos, boss, cache, -beta, -alpha)?;
            best = max(best, score);
            alpha = max(alpha, score);
            if alpha >= beta { break; }
        }

        cache.check_insert(&pos, lower, best, beta);
        ControlFlow::Continue(best)
    }
}

impl Solver<BitBoard> for MinimaxQuick {
    fn solve<M: SolverManager>(pos: BitBoard, boss: &mut M, cache: &mut Cache) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_score();
        let max = pos.will_win_score();
        Self::minimax(pos, boss, cache, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxQuick | BitBoard);
}
