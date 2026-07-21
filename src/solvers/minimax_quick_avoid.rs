use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{BitBoard, Board, CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;

const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

pub fn minimax_quick_avoid<S: SolverManager>(
    pos: BitBoard,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new(hash_map::DEFAULT_SIZE);
    let mut upper = HashMap::new(hash_map::DEFAULT_SIZE);
    let result = minimax_quick_avoid_helper(
        pos,
        boss,
        position::MIN_SCORE,
        position::MAX_SCORE,
        &mut lower,
        &mut upper,
    );
    result
}

pub fn minimax_quick_avoid_helper<S: SolverManager>(
    pos: BitBoard,
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
    debug_assert!(!pos.curr_can_win());

    let Ok(nexts) = pos.possible_nonlosing_nexts() else {
        return ControlFlow::Continue(pos.will_lose_score());
    };

    let moves: MoveSorter<{ column::COUNT }, _, _> = nexts
        .map(|(col, next_board)| (next_board.heuristic(), col))
        .collect();

    alpha = max(alpha, pos.will_lose_score() + 1);
    beta = min(beta, pos.will_win_score() - 1);
    if let Some(min) = lower.get(&pos) {
        alpha = max(alpha, min)
    }
    if let Some(max) = upper.get(&pos) {
        beta = min(beta, max)
    }
    if alpha >= beta {
        return ControlFlow::Continue(beta);
    };

    for col in moves {
        let next_pos = pos.placed_curr_unchecked(col);
        let score = -minimax_quick_avoid_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
        alpha = max(alpha, score);

        if score >= beta {
            if pos.move_count() <= MAX_CACHE_DEPTH {
                lower.insert(&pos, score);
            }
            return ControlFlow::Continue(score);
        }
    }

    if pos.move_count() <= MAX_CACHE_DEPTH {
        upper.insert(&pos, alpha);
    }

    ControlFlow::Continue(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(solve_using(&minimax_quick_avoid), BitBoard);
}
