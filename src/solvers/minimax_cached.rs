use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};

pub struct MinimaxCached { }

impl MinimaxCached {
    fn minimax<P, M>(pos: P, boss: &mut M, cache: &mut Cache) -> ControlFlow<M::Break, isize>
    where
        P: Position + CloneBoard + HashBoard,
        M: SolverManager,
    {
        boss.check()?;
        if pos.completed() { return ControlFlow::Continue(0); }

        if let Some((EntryKind::Exact, score)) = cache.get(&pos) {
            return ControlFlow::Continue(score);
        }

        let mut best = isize::MIN;

        for (col, next_pos) in pos.nexts(pos.curr()) {
            if next_pos.is_won_at_col(col) {
                best = next_pos.just_won_score();
                break;
            }
            let score = -Self::minimax(next_pos, boss, cache)?;
            best = max(best, score);
        }
        cache.insert(EntryKind::Exact, &pos, best);
        ControlFlow::Continue(best)
    }
}

impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxCached {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, cache: &mut Cache) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_score();
        let max = pos.will_win_score();
        Self::minimax(pos, boss, cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxCached | BitCols, BitBoard, SymmBoard);
}
