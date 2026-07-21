use binary_heap_plus::*;
use std::cmp::{max, min};
use std::hash::Hash;
use std::ops::ControlFlow;
use hashbrown::HashMap;

use crate::algorithms::minimax_ordered::minimax_ordered_helper;
use crate::algorithms::minimax_avoidant::minimax_avoidant_helper;
use crate::basic::*;
use crate::board::{Board, CloneBoard, HashBoard, MutBoard};
use crate::algorithms::{Position, SolverManager, position};

pub fn minimax_deepening<P, S, F>(f: &'static F) -> (impl Fn(P, &mut S) -> ControlFlow<S::Break, isize>)
where 
    P: Position + CloneBoard + HashBoard,
    S: SolverManager,
    F: Fn(P, &mut S, isize, isize, &mut HashMap<u64, isize>, &mut HashMap<u64, isize>) -> ControlFlow<S::Break, isize>,

{
    |pos, boss| minimax_deepening_with(pos, boss, f)
}

fn minimax_deepening_with<P, S, F>(pos: P, boss: &mut S, f: &F) -> ControlFlow<S::Break, isize> 
where 
    P: Position + CloneBoard + HashBoard,
    S: SolverManager,
    F: Fn(P, &mut S, isize, isize, &mut HashMap<u64, isize>, &mut HashMap<u64, isize>) -> ControlFlow<S::Break, isize> 
{
    if pos.can_win(pos.curr()) { return ControlFlow::Continue(pos.will_win_score()) }

    let mut lower = HashMap::new();
    let mut upper = HashMap::new();

    boss.check()?;
    let mut min = pos.will_lose_score();
    let mut max = pos.will_win_score();

    while min < max {
        let mut mid = min + (max - min) / 2;
        if mid <= 0 && min/2 < mid { mid = min / 2 };
        if mid >= 0 && max/2 > mid { mid = max / 2};
        let score = f(pos.clone(), boss, mid, mid+1, &mut lower, &mut upper)?;
        if score <= mid { max = score }
        else { min = score };
    }
    
    boss.log_bytes(lower.allocation_size());
    boss.log_bytes(upper.allocation_size());
    ControlFlow::Continue(min)
}

#[cfg(test)]
mod ordered_tests {
    use super::*;
    use crate::algorithms::{minimax_quick_avoid, solve_using};
    use crate::board::*;

    make_solver_tests!(
        solve_using(&minimax_deepening(&minimax_ordered_helper)),
        BitCols,
        BitBoard,
        SymmBoard
    );
}

mod avoidant_tests {
    use super::*;
    use crate::algorithms::{minimax_quick_avoid, solve_using};
    use crate::board::*;
    make_solver_tests!(
        solve_using(&minimax_deepening(&minimax_avoidant_helper)),
        BitCols,
        BitBoard,
        SymmBoard
    );
}
