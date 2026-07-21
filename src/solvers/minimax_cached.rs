use hashbrown::HashMap;
use std::cmp::{max, min};
use std::hash::{Hash, RandomState};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;

pub fn minimax_cached<P: Position + CloneBoard + HashBoard, S: SolverManager>(pos: P, boss: &mut S) -> ControlFlow<S::Break, isize> {
    let mut cache = HashMap::new();
    let result = minimax_cached_helper(pos, boss, &mut cache);
    boss.log_bytes(cache.allocation_size());
    result
}

pub fn minimax_cached_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    cache: &mut HashMap<u64, isize>,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() {
        return ControlFlow::Continue(0);
    }

    if let Some(&cached_result) = cache.get(&pos.key()) {
        return ControlFlow::Continue(cached_result);
    }

    let mut best = isize::MIN;

    for (col, next_pos) in pos.nexts(pos.curr()) {
        if next_pos.is_won_at_col(col) {
            best = next_pos.just_won_score();
            break;
        }
        
        let score = -minimax_cached_helper(next_pos, boss, cache)?;
        best = max(best, score);
    }
    cache.insert(pos.key(), best);
    ControlFlow::Continue(best)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;
    
    make_solver_tests!(solve_using(&minimax_cached), BitCols, BitBoard, SymmBoard);
}
