use crate::basic::*;
use crate::board::{BitCols, HashBoard};
use crate::board::{Board, CloneBoard, MutBoard, bit_col::BitCol};
use crate::solver_utils::Position;
use std::hash::Hash;

/// A newtype over `BitCols` with custom equality and hash
/// so that symmetric boards are equivalent
#[derive(Clone, Debug)]
pub struct SymmBoard(BitCols);

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
        let centre = self.0.get_col(column::Idx::CENTRE).to_u64();

        let mut left = 0; // 012
        for col in column::LEFT_SIDE {
            left = self.0.get_col(col).to_u64() | (left << 7);
        }
        let mut right = 0; // 654
        for col in column::RIGHT_SIDE {
            right = self.0.get_col(col).to_u64() | (right << 7);
        }

        let (left, right) = if left <= right {
            (left, right)
        } else {
            (right, left)
        };
        // 0 1 2 | 3 | 6 5 4
        left | (centre << (3 * 7)) | (right << (4 * 7))
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
            || self.0
                .get_cols()
                .iter()
                .rev()
                .zip(other.0.get_cols().iter())
                .all(|(a, b)| a == b)
    }
}

impl Eq for SymmBoard {}

#[cfg(test)]
mod tests {
    use crate::board::Moves;

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
                token = token.next();
                assert_eq!(board_a, board_b, "Symmetric SymmBoards are not equal");
                assert_eq!(
                    board_a.key(),
                    board_b.key(),
                    "Symmetric SymmBoards have different hashes"
                );
            }
        }
    }

    #[test]
    fn little() {
        let moves_a = Moves::from_string("166");
        let board_a = SymmBoard::from_moves(&moves_a);
        println!("{}", board_a.key());

        let moves_b = Moves::from_string("11444");
        let board_b = SymmBoard::from_moves(&moves_b);
        println!("{}", board_b.key());

    }
}
