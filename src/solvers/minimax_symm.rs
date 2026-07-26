use std::iter::once;

use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard};
use crate::solver_utils::*;
use crate::solvers::minimax_ab_cached::MinimaxABCached;
use crate::solvers::{ABSolver, Solver};

use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::ControlFlow;

/// Type to store the difference in height of each column with its reflection,
/// to efficiently compute when board is symmetrical.
///
/// i.e. symm_diff[i] = column[6-i].height() - column[i].height()
type SymmDiff = [isize; 3];

/// Creates a SymmDiffs for the given board.
/// Returns None if the board is irreversably asymmetrical.
fn make_diffs<B: Board>(board: &B) -> Option<SymmDiff> {
    let mut diffs = [0; 3];
    for i in 0..3 {
        let col_l = column::Idx::try_from(i).unwrap();
        let col_r = col_l.mirrored();
        for row in row::BOTTOM_UP {
            let left = board.get(Cell { col: col_l, row });
            let right = board.get(Cell { col: col_r, row });
            if left != right {
                return None;
            }
        }
        diffs[i] = board.col_count(col_r) as isize - board.col_count(col_l) as isize;
    }

    Some(diffs)
}

pub struct MinimaxSymm { }

impl MinimaxSymm {
    fn minimax<P: Position + CloneBoard + HashBoard, M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache,
        mut alpha: isize,
        mut beta: isize,
        diffs: SymmDiff,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.completed() { return ControlFlow::Continue(0); }

        beta = min(beta, pos.will_win_score());
        (alpha, beta) = cache.check(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        let lower = alpha;
        let mut best = isize::MIN;
        for (diffs, col, next_pos) in next_boards(&pos, diffs) {
            if next_pos.is_won_at_col(col) {
                best = next_pos.just_won_score();
                break;
            }

            let score = -match diffs {
                None => MinimaxABCached::minimax(next_pos, boss, cache, -beta, -alpha)?,
                Some(diffs) => Self::minimax(next_pos, boss, cache, -beta, -alpha, diffs)?,
            };
            best = max(best, score);
            alpha = max(alpha, score);
            if alpha >= beta { break; }
        }

        cache.check_insert(&pos, lower, best, beta);
        ControlFlow::Continue(best)
    }
}

impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxSymm {
    fn solve<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache,
    ) -> ControlFlow<M::Break, isize> {
        let alpha = pos.will_lose_score();
        let beta = pos.will_win_score();
        match make_diffs(&pos) {
            None => MinimaxABCached::solve(pos, boss, cache),
            Some(diffs) => Self::minimax(pos, boss, cache, alpha, beta, diffs),
        }
    }
}

/// Updates the given diff considering the token from the prev move.
/// Returns None if the board is irreversibly asymmetrical
fn next_diffs<B: Board>(board: &B, col: column::Idx, diffs: SymmDiff) -> Option<SymmDiff> {
    if col == column::Idx::CENTRE {
        return Some(diffs);
    }

    let mut new_diffs = diffs;
    let cell = board.top(col).unwrap();
    let token = board.get(cell)?;
    let cell_c = cell.mirrored();

    if let Some(token_c) = board.get(cell_c)
        && token != token_c
    {
        return None;
    }

    if cell.col.is_left_side() {
        new_diffs[usize::from(cell.col)] -= 1;
    } else {
        new_diffs[usize::from(cell_c.col)] += 1;
    }

    Some(new_diffs)
}

fn next_boards<P: Position + CloneBoard>(
    pos: &P,
    diffs: SymmDiff,
) -> Vec<(Option<SymmDiff>, column::Idx, P)> {
    match diffs {
        [0, 0, 0] => column::LEFT_SIDE
            .into_iter()
            .chain(once(column::Idx::CENTRE))
            .filter_map(|col| {
                let pos = pos.placed(col, pos.curr())?;
                Some((col, pos))
            })
            .map(|(col, pos)| (next_diffs(&pos, col, diffs), col, pos))
            .collect(),
        _ => pos
            .nexts(pos.curr())
            .map(|(col, pos)| (next_diffs(&pos, col, diffs), col, pos))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxSymm | BitCols, SymmBoard, BitBoard);
}
