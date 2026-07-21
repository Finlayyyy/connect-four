use binary_heap_plus;
use hashbrown::HashMap;
use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::ControlFlow;
use arrayvec::ArrayVec;
use heapless::binary_heap::{BinaryHeap, Max};

use crate::algorithms::move_sorter::MoveSorter;
use crate::algorithms::{Position, SolverManager, position};
use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard, MutBoard};

pub fn minimax_avoidant<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut lower = HashMap::new();
    let mut upper = HashMap::new();
    let result = minimax_avoidant_helper(
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

#[derive(Clone, Copy, Debug)]
/// (curr_triples, opp_triples, curr_pairs, opp_pairs)
struct NbhdCounts(usize, usize, usize, usize);
impl NbhdCounts {
    fn heuristic(&self) -> usize {
        self.0 * 1000
        + self.1 * 100
        + self.2 * 10
        + self.3
    }
}

#[derive(Clone, Debug)]
enum MoveResult {
    CurrWin,        // connect-4 for curr
    BlockOppWin,    // stop opp making connect-4
    LetOppWin,      // opp can now play on top of curr's token and connect-4
    ForcedOppWin, // opp has a potential connect-4 both at the current cell and the one above, curr has lost
    Nbhd(NbhdCounts)
}

impl NbhdCounts {
    fn new_at<P: Position + Board >(pos: &P, cell: Cell) -> MoveResult {
        let Some((curr_pairs, curr_triples)) = pos.count_adjacent_at(cell, pos.curr()) else {
            return MoveResult::CurrWin;
        };

        let Some((opp_pairs, opp_triples)) = pos.count_adjacent_at(cell, pos.opp()) else {
            if let Some(above) = cell.above() {
                if pos.count_adjacent_at(above, pos.opp()).is_none() {
                    return MoveResult::ForcedOppWin;
                }
            }
            return MoveResult::BlockOppWin;
        };

        if let Some(above) = cell.above() {
            if pos.count_adjacent_at(above, pos.opp()).is_none() {
                return MoveResult::LetOppWin;
            }
        }

        MoveResult::Nbhd(NbhdCounts(curr_triples, opp_triples, curr_pairs, opp_pairs))
    }
}


pub fn minimax_avoidant_helper<P: Position + CloneBoard + HashBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
    mut alpha: isize,
    mut beta: isize,
    lower: &mut HashMap<u64, isize>,
    upper: &mut HashMap<u64, isize>,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;
    if pos.completed() { return ControlFlow::Continue(0) };

    if let Some(&min) = lower.get(&pos.key()) { alpha = max(alpha, min) };
    if let Some(&max) = upper.get(&pos.key()) { beta = min(beta, max) };
    if alpha >= beta { return ControlFlow::Continue(beta) };

    let mut moves = MoveSorter::<{column::COUNT}, _, _>::new();
    let mut must_play = None;
    let mut must_avoid = None;
    let mut will_lose = false;

    for cell in pos.next_cells() {
        match NbhdCounts::new_at(&pos, cell) {
            // curr has at least one immediately winning move
            MoveResult::CurrWin => return ControlFlow::Continue(pos.will_win_score()),
            // opponent has at least one winning move next turn
            MoveResult::BlockOppWin if must_play.is_none() => must_play = Some(cell.col),
            // opponent has at least two winning moves next turn, thus curr has lost.
            MoveResult::BlockOppWin | MoveResult::ForcedOppWin => will_lose = true,
            // Playing this move will allow opponent to win
            MoveResult::LetOppWin => must_avoid = Some(cell.col),
            // there are no immediate wins or losses
            MoveResult::Nbhd(nbhd) => moves.push_sorting(nbhd.heuristic(), cell.col)
        }
    }
    if will_lose { return ControlFlow::Continue(pos.will_lose_score()) };
    if moves.is_empty() && must_play.is_none() { must_play = must_avoid };

    // We must play in this column either to stop the opponent from winning,
    // or because it is the only option left
    if let Some(col) = must_play { 
        let next_pos = pos.placed(col, pos.curr()).unwrap();
        
        let score = -minimax_avoidant_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
        alpha = max(alpha, score);
        return ControlFlow::Continue(score);
    }

    alpha = max(alpha, pos.will_lose_score() + 1);
    beta = min(beta, pos.will_win_score() - 1);
    if alpha >= beta { return ControlFlow::Continue(beta) };

    for col in moves {
        let next_pos = pos.placed(col, pos.curr()).unwrap();
        let score = -minimax_avoidant_helper(next_pos, boss, -beta, -alpha, lower, upper)?;
        alpha = max(alpha, score);

        if score >= beta { 
            lower.insert(pos.key(), score);
            return ControlFlow::Continue(score);
        }
    }

    upper.insert(pos.key(), alpha);
    ControlFlow::Continue(alpha)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithms::solve_using;
    use crate::board::*;

    make_solver_tests!(
        solve_using(&minimax_avoidant),
        BitCols,
        BitBoard,
        SymmBoard
    );

}
