use crate::solvers::minimax_cached::minimax_cached_helper;
use crate::solver_utils::*;
use crate::solvers::minimax_ab_cached::minimax_ab_cached_helper;
use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard};
use hashbrown::HashMap;
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
        let col_r = col_l.reflected();
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

pub fn minimax_symm<P: Position + CloneBoard + HashBoard, S: SolverManager>(pos: P, boss: &mut S) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new();
    let mut upper = HashMap::new();
    let result = if let Some(diffs) = make_diffs(&pos) {
        minimax_symm_helper(pos, boss, position::MIN_SCORE, position::MAX_SCORE, &mut lower, &mut upper, diffs)
    } else {
        minimax_ab_cached_helper(pos, boss, position::MIN_SCORE, position::MAX_SCORE, &mut lower, &mut upper)
    };
    boss.log_bytes(lower.allocation_size());
    boss.log_bytes(upper.allocation_size());
    result
}

/// Updates the given diff considering the token from the prev move.
/// Returns None if the board is irreversibly asymmetrical
fn next_diffs<B: Board>(board: &B, col: column::Idx, diffs: SymmDiff) -> Option<SymmDiff> {
    if col == column::Idx::CENTRE { return Some(diffs) }

    let mut new_diffs = diffs;
    let cell = board.top(col).unwrap();
    let token = board.get(cell)?;
    let cell_c = cell.reflected();

    if let Some(token_c) = board.get(cell_c) && token != token_c { return None; }

    if cell.col.is_left() {
        new_diffs[usize::from(cell.col)] -= 1;
    } else {
        new_diffs[usize::from(cell_c.col)] += 1;
    }

    Some(new_diffs)
}

fn next_boards<P: Position + CloneBoard>(pos: &P, diffs: SymmDiff) -> Vec<(Option<SymmDiff>, column::Idx, P)> {
    match diffs {
        [0, 0, 0] => (column::LEFT..=column::CENTRE)
            .filter_map(|col| {
                let pos = pos.placed(col, pos.curr())?;
                Some((col, pos))
            })
            .map(|(col, pos)| (next_diffs(&pos, col, diffs), col, pos))
            .collect(),
        _ => pos
            .nexts(pos.curr())
            .map(|(col, pos)| (next_diffs(&pos, col, diffs), col, pos))
            .collect()
    }
}

fn minimax_symm_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    mut alpha: isize,
    mut beta: isize,
    lower: &mut HashMap<u64, isize>,
    upper: &mut HashMap<u64, isize>,
    diffs: SymmDiff,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() { return ControlFlow::Continue(0) };

    beta = min(beta, pos.will_win_score());
    if let Some(&min) = lower.get(&pos.key()) { alpha = max(alpha, min) };
    if let Some(&max) = upper.get(&pos.key()) { beta = min(beta, max) };
    if alpha >= beta { return ControlFlow::Continue(beta) };

    for (diffs, col, next_pos) in next_boards(&pos, diffs) {
        if next_pos.is_won_at_col(col) {
            alpha = next_pos.just_won_score();
            break;
        }

        let score = -match diffs {
            None => minimax_ab_cached_helper(next_pos, boss, -beta, -alpha, lower, upper)?,
            Some(diffs) => minimax_symm_helper(next_pos, boss, -beta, -alpha, lower, upper, diffs)?,
        };

        if score >= beta { 
            lower.insert(pos.key(), score);
            return ControlFlow::Continue(score);
        }
        alpha = max(alpha, score);
    }

    upper.insert(pos.key(), alpha);
    ControlFlow::Continue(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(
        &solve_using(&minimax_symm),
        BitCols,
        SymmBoard,
        BitBoard
    );


}
