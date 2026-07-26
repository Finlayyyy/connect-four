use std::cmp::{max, min};
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;

pub fn minimax_ordered<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new(hash_map::LARGE_SIZE);
    let mut upper = HashMap::new(hash_map::LARGE_SIZE);
    let alpha = pos.will_lose_score();
    let beta = pos.will_win_score();
    let result = minimax_ordered_helper(pos, boss, alpha, beta, &mut lower, &mut upper);
    result
}

pub fn minimax_ordered_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
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

    if let Some(low) = lower.get(&pos) {
        alpha = max(alpha, low)
    };
    if let Some(up) = upper.get(&pos) {
        beta = min(beta, up)
    };
    if alpha >= beta {
        return ControlFlow::Continue(beta);
    };

    let mut moves = MoveSorter::<{ column::COUNT }, _, _>::new();

    for (col, next_pos) in pos.nexts(pos.curr()) {
        let cell = next_pos.top(col).unwrap();
        match next_pos.count_adjacent_around(cell, pos.curr()) {
            // found a win
            None => {
                let score = next_pos.just_won_score();
                // upper.insert(pos.board().clone(), score);
                // lower.insert(pos.board().clone(), score);
                return ControlFlow::Continue(score);
            }
            // add the board
            Some(adjs) => moves.push_sorting(adjs, (col, next_pos)),
        }
    }
    alpha = max(alpha, pos.will_lose_score());
    beta = min(beta, pos.will_win_score() - 1);
    if alpha >= beta {
        return ControlFlow::Continue(beta);
    };

    for (col, next_pos) in moves {
        let score = -minimax_ordered_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
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

    make_solver_tests!(solve_using(&minimax_ordered), BitCols, BitBoard, SymmBoard);
}
