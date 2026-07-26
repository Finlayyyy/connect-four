use crate::basic::*;
use crate::board::{BitCols, HashBoard};
use crate::board::{Board, CloneBoard, MutBoard, bit_col::BitCol};
use crate::solver_utils::Position;
use std::hash::Hash;

/// A board implementation using bit manipulation for storage with
/// customised equality and hashing for symmetry.
/// Each column is stored as a BitCol.
#[derive(Clone, Debug)]
pub struct SymmBoard(BitCols);

impl SymmBoard {
    fn get_col(&self, col: column::Idx) -> BitCol {
        self.0.get_col(col)
    }
    fn get_cols(&self) -> &[BitCol] {
        self.0.get_cols()
    }
}

impl Board for SymmBoard {
    const EMPTY: Self = SymmBoard(BitCols::EMPTY);

    fn get(&self, cell: Cell) -> Option<Token> {
        self.0.get(cell)
    }
    fn count_moves(&self) -> usize {
        self.0.count_moves()
    }
    fn col_count(&self, col: column::Idx) -> usize {
        self.0.col_count(col)
    }
    fn can_place(&self, col: column::Idx) -> bool {
        self.0.can_place(col)
    }
    fn force_place(&mut self, col: column::Idx, token: Token) {
        self.0.force_place(col, token)
    }
    fn place(&mut self, col: column::Idx, token: Token) -> Option<Cell> {
        self.0.place(col, token)
    }
}

impl CloneBoard for SymmBoard {}

impl MutBoard for SymmBoard {
    fn unplace(&mut self, col: column::Idx) {
        self.0.unplace(col)
    }
}

impl HashBoard for SymmBoard {
    fn key(&self) -> u64 {
        let centre = self.get_col(column::Idx::CENTRE).to_u64();

        let mut left = 0;
        for col in column::LEFT_SIDE {
            left = self.get_col(col).to_u64() | (left << 8);
        }
        let mut right = 0;
        for col in column::RIGHT_SIDE.into_iter().rev() {
            right = self.get_col(col).to_u64() | (right << 8);
        }

        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };

        left | (centre << (3 * 8)) | (right << (4 * 8))
    }
}

impl Position for SymmBoard {
    fn move_count(&self) -> usize {
        self.0.move_count()
    }
    fn curr(&self) -> Token {
        self.0.curr()
    }
}

impl PartialEq for SymmBoard {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
            || self
                .get_cols()
                .iter()
                .rev()
                .zip(other.get_cols().iter())
                .all(|(a, b)| a == b)
    }
}

impl Eq for SymmBoard {}

#[cfg(test)]
mod tests {
    use std::hash::DefaultHasher;
    use std::hash::Hasher;

    use super::*;

    make_board_tests!(SymmBoard);
    make_mut_board_tests!(SymmBoard);

    #[test]
    fn test_symmetry() {
        let mut board_a = SymmBoard::EMPTY;
        let mut board_b = SymmBoard::EMPTY;
        let mut token = Token::STARTING;
        for _ in row::BOTTOM_UP {
            for col in column::COLUMNS {
                board_a.place(col, token).unwrap();
                board_b.place(col.mirrored(), token).unwrap();

                assert_eq!(board_a, board_b, "Symmetric SymmBoards are not equal");
                assert_eq!(
                    board_a.key(),
                    board_b.key(),
                    "Symmetric SymmBoards have different hashes"
                );
            }
        }
    }
}
