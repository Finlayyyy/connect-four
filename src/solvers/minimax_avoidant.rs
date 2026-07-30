use arrayvec::ArrayVec;
use heapless::binary_heap::{BinaryHeap, Max};
use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::ControlFlow;

use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard, MutBoard};
use crate::solver_utils::*;
use crate::solvers::{Solver, ABSolver};

/// Struct to hold counts of adjacent cells for use in heuristic evaluation
/// (curr_triples, opp_triples, curr_pairs, opp_pairs)
#[derive(Clone, Copy, Debug)]
struct NbhdCounts(usize, usize, usize, usize);
impl NbhdCounts {
    /// Returns a heuristic value for the current state of adjacent cells
    pub fn heuristic(&self) -> usize {
        self.0 * 10_000 + self.3 * 1000 + 100 - (self.2 * 10 + self.3)
    }
}

/// Enum representing the result of a move
#[derive(Clone, Debug)]
enum MoveResult {
    CurrWin,      // connect-4 for curr
    BlockOppWin,  // stop opp making connect-4
    LetOppWin,    // opp can now play on top of curr's token and connect-4
    ForcedOppWin, // opp has a potential connect-4 both at the current cell and the one above, curr has lost
    Nbhd(NbhdCounts), // no immediate win for curr or opp
}

impl MoveResult {
    /// Returns the result of a move at the given cell
    fn new_at<P: Position + Board>(pos: &P, cell: Cell) -> MoveResult {
        let Some((curr_pairs, curr_triples)) = pos.count_adjacent_around(cell, pos.curr()) else {
            return MoveResult::CurrWin;
        };

        let Some((opp_pairs, opp_triples)) = pos.count_adjacent_around(cell, pos.opp()) else {
            if let Some(above) = cell.above()
                && pos.count_adjacent_around(above, pos.opp()).is_none() {
                    return MoveResult::ForcedOppWin;
            }
            return MoveResult::BlockOppWin;
        };

        if let Some(above) = cell.above()
            && pos.count_adjacent_around(above, pos.opp()).is_none() {
                return MoveResult::LetOppWin;
        }

        MoveResult::Nbhd(NbhdCounts(curr_triples, opp_triples, curr_pairs, opp_pairs))
    }
}

/// Extends `MinimaxOrdered` by avoiding immediate enemy wins
pub struct MinimaxAvoidant { }
impl<P: Position + CloneBoard + HashBoard> ABSolver<P> for MinimaxAvoidant {
    fn minimax<M: SolverManager>(
        pos: P,
        boss: &mut M,
        cache: &mut Cache<P>,
        mut alpha: isize,
        mut beta: isize,
    ) -> ControlFlow<M::Break, isize> {
        boss.check()?;
        if pos.full() { return ControlFlow::Continue(0); }
        let prev_alpha = alpha;

        (alpha, beta) = cache.get_check(&pos, alpha, beta);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        let mut moves = MoveSorter::<{ column::COUNT }, _, _>::new();
        let mut must_play = None;
        let mut will_lose = false;

        for cell in pos.next_cells() {
            match MoveResult::new_at(&pos, cell) {
                // curr has at least one immediately winning move
                MoveResult::CurrWin => return ControlFlow::Continue(pos.will_win_score()),
                // opponent has at least one winning move next turn
                MoveResult::BlockOppWin if must_play.is_none() => must_play = Some(cell.col),
                // opponent has at least two winning moves next turn, thus curr has lost.
                MoveResult::BlockOppWin | MoveResult::ForcedOppWin => will_lose = true,
                // Playing this move will allow opponent to win
                MoveResult::LetOppWin => (),
                // there are no immediate wins or losses
                MoveResult::Nbhd(nbhd) => moves.push_sorting(nbhd.heuristic(), cell.col),
            }
        }

        if will_lose || (moves.is_empty() && must_play.is_none()) {
            return ControlFlow::Continue(pos.will_lose_score());
        };

        // We must play in this column either to stop the opponent from winning,
        // or because it is the only option left
        if let Some(col) = must_play {
            moves = MoveSorter::singleton(0, col);
        }

        alpha = max(alpha, pos.will_lose_score() + 1);
        beta = min(beta, pos.will_win_score() - 1);
        if alpha >= beta { return ControlFlow::Continue(beta); }

        for col in moves {
            let next_pos = pos.placed(col, pos.curr()).unwrap();
            let score = -Self::minimax(next_pos, boss, cache, -beta, -alpha)?;
            alpha = max(alpha, score);
            if alpha >= beta { break; }
        }

        cache.insert_check(&pos, prev_alpha, alpha, beta);
        ControlFlow::Continue(alpha)
    }
}
impl<P: Position + CloneBoard + HashBoard> Solver<P> for MinimaxAvoidant {
    fn solve<M: SolverManager>(pos: P, boss: &mut M, cache: &mut Cache<P>) -> ControlFlow<M::Break, isize> {
        let min = pos.will_lose_score();
        let max = pos.will_win_score();
        Self::minimax(pos, boss, cache, min, max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(MinimaxAvoidant | BitCols, BitBoard, SymmBoard);
}
