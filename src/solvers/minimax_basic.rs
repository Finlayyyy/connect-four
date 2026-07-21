use std::cmp::max;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::basic::*;
use crate::board::{CloneBoard, MutBoard};
use crate::solver_utils::*;

pub fn minimax_mut<P: Position + MutBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    let mut pos = pos;
    minimax_mut_helper(&mut pos, boss)
}

fn minimax_mut_helper<P: Position + MutBoard, S: SolverManager>(
    pos: &mut P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;

    if pos.completed() {
        return ControlFlow::Continue(0);
    }

    let mut best_score = isize::MIN;

    for col in column::CENTRED {
        let Some(cell) = pos.place_curr(col) else {
            continue;
        };
        if pos.is_won_at(cell) {
            let score = pos.just_won_score();
            pos.unplace(cell);
            return ControlFlow::Continue(score);
        }

        let score = -minimax_mut_helper(pos, boss)?;
        pos.unplace(cell);
        best_score = max(best_score, score);
    }

    ControlFlow::Continue(best_score)
}

pub fn minimax_clone<P: Position + CloneBoard, S: SolverManager>(
    pos: P,
    boss: &mut S,
) -> ControlFlow<S::Break, isize> {
    boss.check()?;

    if pos.completed() {
        return ControlFlow::Continue(0);
    }

    let mut best_score = isize::MIN;

    for (col, next_pos) in pos.nexts(pos.curr()) {
        if next_pos.is_won_at_col(col) {
            return ControlFlow::Continue(next_pos.just_won_score());
        }

        let score = -minimax_clone(next_pos, boss)?;
        best_score = max(best_score, score);
    }
    ControlFlow::Continue(best_score)
}

#[cfg(test)]
mod mut_tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(solve_using(&minimax_mut), BitCols, BitBoard, SymmBoard);
}

#[cfg(test)]
mod clone_tests {
    use super::*;
    use crate::solvers::testing::*;

    make_solver_tests!(solve_using(&minimax_clone), BitCols, BitBoard, SymmBoard);
}
