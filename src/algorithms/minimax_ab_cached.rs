use hashbrown::HashMap;
use std::cmp::{max, min};
use std::hash::{Hash, RandomState};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard, MutBoard};
use crate::algorithms::{Position, SolverManager, position};

pub fn minimax_ab_cached<P: Position + CloneBoard + HashBoard, S: SolverManager>(pos: P, boss: &mut S) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new();
    let mut upper = HashMap::new();
    let result = minimax_ab_cached_helper(pos, boss, position::MIN_SCORE, position::MAX_SCORE, &mut lower, &mut upper);
    boss.log_bytes(lower.allocation_size());
    boss.log_bytes(upper.allocation_size());
    result
}

pub fn minimax_ab_cached_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    mut alpha: isize, 
    mut beta: isize,
    lower: &mut HashMap<u64, isize>,
    upper: &mut HashMap<u64, isize>
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() { return ControlFlow::Continue(0) };

    beta = min(beta, pos.will_win_score());
    if let Some(&min) = lower.get(&pos.key()) { alpha = max(alpha, min) };
    if let Some(&max) = upper.get(&pos.key()) { beta = min(beta, max) };
    if (alpha >= beta) { return ControlFlow::Continue(beta) };

    for (col, next_pos) in pos.nexts(pos.curr()) {
        if next_pos.is_won_at_col(col) {
            alpha = next_pos.just_won_score();
            break;
        }
        
        let score = -minimax_ab_cached_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
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
    use crate::algorithms::solve_using;
    use crate::board::*;

    make_solver_tests!(solve_using(&minimax_ab_cached), BitCols, BitBoard, SymmBoard);
}
