use std::ops::{Index, IndexMut};

use crate::basic::*;
use crate::board::{Board, CloneBoard, MutBoard};
use crate::solver_utils::Position;

/// MutBoard implementation using a 2D array of Option<Token>.
/// An array of columns, where the 0th element is the bottom of the column.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayBoard {
    grid: [[Option<Token>; row::COUNT]; column::COUNT],
}

impl Index<Cell> for ArrayBoard {
    type Output = Option<Token>;

    fn index(&self, index: Cell) -> &Self::Output {
        &self.grid[usize::from(index.col)][usize::from(index.row)]
    }
}

impl IndexMut<Cell> for ArrayBoard {
    fn index_mut(&mut self, index: Cell) -> &mut Self::Output {
        &mut self.grid[usize::from(index.col)][usize::from(index.row)]
    }
}

impl Board for ArrayBoard {
    const EMPTY: Self = ArrayBoard {
        grid: [[None; row::COUNT]; column::COUNT],
    };

    fn can_place(&self, col: column::Idx) -> bool {
        self[Cell {
            col: col,
            row: row::Idx::TOP,
        }]
        .is_none()
    }

    fn get(&self, cell: Cell) -> Option<Token> {
        self[cell]
    }

    fn place(&mut self, col: column::Idx, token: Token) -> Option<Cell> {
        for row in row::BOTTOM_UP {
            if self[Cell { col, row }].is_none() {
                self[Cell { col, row }] = Some(token);
                return Some(Cell { col, row });
            }
        }
        None
    }
}

impl MutBoard for ArrayBoard {
    fn unplace(&mut self, cell: Cell) {
        self[cell] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    make_board_tests!(ArrayBoard);

    make_mut_board_tests!(ArrayBoard);
}

#[cfg(test)]
mod pos_tests {
    use super::*;
    use crate::solver_utils::*;

    make_board_tests!(WithInfo<ArrayBoard>);

    make_mut_board_tests!(WithInfo<ArrayBoard>);
}
