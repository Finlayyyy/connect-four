use std::cmp::{max, min};
use std::hash::{Hash, RandomState};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard, MutBoard};

use crate::solver_utils::*;

pub fn minimax_ab_cached<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new(hash_map::LARGE_SIZE);
    let mut upper = HashMap::new(hash_map::LARGE_SIZE);
    let result = minimax_ab_cached_helper(
        pos,
        boss,
        position::MIN_SCORE,
        position::MAX_SCORE,
        &mut lower,
        &mut upper,
    );
    result
}

pub fn minimax_ab_cached_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    mut alpha: isize,
    mut beta: isize,
    lower: &mut HashMap,
    upper: &mut HashMap,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() {
        return ControlFlow::Continue(0);
    };

    beta = min(beta, pos.will_win_score());
    if let Some(min) = lower.get(&pos) {
        alpha = max(alpha, min)
    };
    if let Some(max) = upper.get(&pos) {
        beta = min(beta, max)
    };
    if (alpha >= beta) {
        return ControlFlow::Continue(beta);
    };

    for (col, next_pos) in pos.nexts(pos.curr()) {
        if next_pos.is_won_at_col(col) {
            alpha = next_pos.just_won_score();
            break;
        }

        let score = -minimax_ab_cached_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
        if score >= beta {
            lower.insert(&pos, score);
            return ControlFlow::Continue(score);
        }
        alpha = max(alpha, score);
    }
    upper.insert(&pos, alpha);
    ControlFlow::Continue(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(
        solve_using(&minimax_ab_cached),
        BitCols,
        BitBoard,
        SymmBoard
    );
}
