use binary_heap_plus;
use hashbrown::HashMap;
use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::ControlFlow;
use arrayvec::ArrayVec;
use heapless::binary_heap::{BinaryHeap, Max};

use crate::solver_utils::*;
use crate::basic::*;
use crate::board::{BitBoard, Board, CloneBoard, HashBoard, MutBoard};

const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

pub fn minimax_quick_avoid<S: SolverManager>(
    pos: BitBoard,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new();
    let mut upper = HashMap::new();
    let result = minimax_quick_avoid_helper(
        pos,
        boss,
        position::MIN_SCORE,
        position::MAX_SCORE,
        &mut lower,
        &mut upper,
    );
    boss.log_bytes(lower.allocation_size());
    boss.log_bytes(upper.allocation_size());
    result
}

pub fn minimax_quick_avoid_helper<S: SolverManager>(
    pos: BitBoard,
    boss: &mut S,
    mut alpha: isize,
    mut beta: isize,
    lower: &mut HashMap<u64, isize>,
    upper: &mut HashMap<u64, isize>,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() { return ControlFlow::Continue(0) };
    debug_assert!(!pos.curr_can_win());

    let Ok(nexts) = pos.possible_nonlosing_nexts() else {
        return ControlFlow::Continue(pos.will_lose_score());
    };

    let moves: MoveSorter::<{column::COUNT}, _, _>  = nexts
        .map(|(col, next_board)| (next_board.heuristic(), col))
        .collect();

    alpha = max(alpha, pos.will_lose_score() + 1);
    beta = min(beta, pos.will_win_score() - 1);
    if let Some(&min) = lower.get(&pos.key()) { alpha = max(alpha, min) };
    if let Some(&max) = upper.get(&pos.key()) { beta = min(beta, max) };
    if alpha >= beta { return ControlFlow::Continue(beta) };

    for col in moves {
        let next_pos = pos.placed_curr_unchecked(col);
        let score = -minimax_quick_avoid_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
        alpha = max(alpha, score);

        if score >= beta { 
            if pos.move_count() <= MAX_CACHE_DEPTH {
                lower.insert(pos.key(), score);
            }
            return ControlFlow::Continue(score);
        }
    }
    
    if pos.move_count() <= MAX_CACHE_DEPTH {
        upper.insert(pos.key(), alpha);
    }
    
    ControlFlow::Continue(alpha)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(
        solve_using(&minimax_quick_avoid),
        BitBoard
    );

}
