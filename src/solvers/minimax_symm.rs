use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard};
use crate::solver_utils::*;
use crate::solvers::minimax_ab_cached::MinimaxABCached;
use crate::solvers::{ABSolver, Solver};

use std::cmp::{max, min};
use std::ops::ControlFlow;

/// Solver that extends `MinimaxABCached` to use symmetry to
/// reduce the search space
pub struct MinimaxSymm { }

impl MinimaxSymm {
    fn minimax<P: Position + CloneBoard + HashBoard, M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache<P>,
        mut alpha: isize,
        mut beta: isize,
        diffs: SymmDiff,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.is_full() { return ControlFlow::Continue(0); }
        let prev_alpha = alpha;

        beta = min(beta, pos.eval());
        (alpha, beta) = cache.get_and_bound(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        for (diffs, col, next_pos) in diffs.nexts(&pos) {
            if next_pos.is_won_at_col(col) {
                alpha = next_pos.just_won_eval();
                break;
            }

            let eval = -match diffs {
                None => MinimaxABCached::minimax(next_pos, boss, cache, -beta, -alpha)?,
                Some(diffs) => Self::minimax(next_pos, boss, cache, -beta, -alpha, diffs)?,
            };
            alpha = max(alpha, eval);
            if alpha >= beta { break; }
        }

        cache.insert_bounded(&pos, prev_alpha, alpha, beta);
        ControlFlow::Continue(alpha)
    }
}

impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxSymm {
    fn solve<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache<P>,
    ) -> ControlFlow<M::Break, isize> {
        let alpha = pos.will_lose_eval();
        let beta = pos.eval();
        match SymmDiff::new(&pos) {
            None => MinimaxABCached::solve(pos, boss, cache),
            Some(diffs) => Self::minimax(pos, boss, cache, alpha, beta, diffs),
        }
    }
}


/// Type to store the difference in height of each column with its reflection,
/// to efficiently compute when a board is symmetrical.
///
/// i.e. symm_diff[i] = column[6-i].height() - column[i].height()
#[derive(Debug, Clone)]
struct SymmDiff([isize; 3]);

impl SymmDiff {
    /// Creates a SymmDiffs for the given board.
    /// Returns None if the board is irreversably asymmetrical.
    pub fn new<B: Board>(board: &B) -> Option<Self> {
        let mut diffs = [0; 3];
        for col_l in column::LEFT_SIDE {
            let col_r = col_l.mirrored();
            for row in row::BOTTOM_UP {
                let left = board.get(Cell { col: col_l, row });
                let right = board.get(Cell { col: col_r, row });
                if left != right {
                    return None;
                }
            }
            let count_l = board.col_count(col_l) as isize;
            let count_r = board.col_count(col_r) as isize;
            diffs[usize::from(col_l)] = count_r - count_l;
        }

        Some(SymmDiff(diffs))
    }

    /// Updates the given diff considering the token from the prev move.
    /// Returns None if the board is irreversibly asymmetrical
    pub fn next<B: Board>(self, board: &B, col: column::Idx) -> Option<Self> {
        if col == column::Idx::CENTRE {
            return Some(self);
        }

        let mut new_diffs = self;
        let cell = board.top(col).unwrap();
        let token = board.get(cell)?;
        let cell_c = cell.mirrored();

        if let Some(token_c) = board.get(cell_c) && token != token_c {
            return None;
        }

        if cell.col.is_left_side() {
            new_diffs.0[usize::from(cell.col)] -= 1;
        } else {
            new_diffs.0[usize::from(cell_c.col)] += 1;
        }

        Some(new_diffs)
    }

    /// Returns the next positions from the given position,
    /// with their corresponding `SymmDiff`s
    fn nexts<P: Position + CloneBoard>(self, pos: &P) -> Vec<(Option<SymmDiff>, column::Idx, P)> {
        match self.0 {
            [0, 0, 0] => column::LEFT_SIDE
                .into_iter()
                .chain(std::iter::once(column::Idx::CENTRE))
                .filter_map(|col| {
                    let pos = pos.placed(col, pos.curr())?;
                    Some((col, pos))
                })
                .map(|(col, pos)| (self.clone().next(&pos, col), col, pos))
                .collect(),
            _ => pos
                .nexts(pos.curr())
                .map(|(col, pos)| (self.clone().next(&pos, col), col, pos))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxSymm | BitCols, SymmBoard, BitBoard);
}
