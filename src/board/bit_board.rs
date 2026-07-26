use crate::basic::*;
use crate::board::HashBoard;
use crate::board::{Board, CloneBoard, MutBoard, bit_col::BitCol};
use crate::solver_utils::Position;

use std::fmt::Debug;

//   6 13 20 27 34 41 48
//  ---------------------
// | 5 12 19 26 33 40 47 |
// | 4 11 18 25 32 39 46 |
// | 3 10 17 24 31 38 45 |
// | 2  9 16 23 30 37 44 |
// | 1  8 15 22 29 36 43 |
// | 0  7 14 21 28 35 42 |
//  ---------------------
/// A fast BitBoard implementation.
/// Stores two bitmaps
/// - board: bitmap of the current player's tokens
/// - mask: bitmap of placed tokens
///
/// Allows for fast querying, hashing and win checks.
#[derive(Clone, PartialEq, Eq)]
pub struct BitBoard {
    board: u64,
    mask: u64,
}

/// Width of the board
const WIDTH: u64 = column::COUNT as u64;

/// Bitmask of the bottom row
const BOTTOM_MASK: u64 = {
    let mut mask: u64 = 0;
    let mut i = 0;
    while i < column::COUNT {
        mask |= 1 << (WIDTH * i as u64);
        i += 1;
    }
    mask
};

/// Bitmask of the additional row above
const ABOVE_MASK: u64 = BOTTOM_MASK << 6;

/// Bitmask of every valid board position
const BOARD_MASK: u64 = ((1 << (WIDTH * WIDTH - 1)) - 1) & (!ABOVE_MASK);

/// Bitmask of the given row
#[inline(always)]
const fn row_mask(row: row::Idx) -> u64 {
    BOTTOM_MASK << row.to_u64()
}

/// Bitmask of the cell at the bottom of the given cell
#[inline(always)]
const fn bottom_col_mask(col: column::Idx) -> u64 {
    1 << (WIDTH * col.to_u64())
}

/// Bitmask of the given column
#[inline(always)]
const fn col_mask(col: column::Idx) -> u64 {
    0b111111 << (WIDTH * col.to_u64())
}

/// Bitmask fo the given cell
#[inline(always)]
const fn cell_mask(cell: Cell) -> u64 {
    (1 << cell.row.to_u64()) << (WIDTH * cell.col.to_u64())
}

impl BitBoard {
    /// Bitmask with a 1 at the cell of every valid move
    #[inline(always)]
    fn possible_mask(&self) -> u64 {
        (self.mask + BOTTOM_MASK) & BOARD_MASK
    }

    /// Place the current player's token at the given column,
    /// without checking if it is not full.
    #[inline(always)]
    pub fn placed_curr_unchecked(&self, col: column::Idx) -> Self {
        BitBoard {
            board: self.board ^ self.mask,
            mask: self.mask | (self.mask + bottom_col_mask(col)),
        }
    }

    /// Bitmask with a 1 at every cell that would result
    /// in a win for the current player
    fn curr_win_mask(&self) -> u64 {
        Self::win_mask(self.board, self.mask)
    }
    /// Bitmask with a 1 at every cell that would result
    /// in a win for the opponent
    fn opp_win_mask(&self) -> u64 {
        Self::win_mask(self.board ^ self.mask, self.mask)
    }

    /// Bitmask of all current winning moves
    fn win_mask(board: u64, mask: u64) -> u64 {
        // Vertical
        let mut r = (board << 1) & (board << 2) & (board << 3);

        // Horizontal
        let mut p = (board << WIDTH) & (board << (2 * WIDTH)); // 0 1 1 => 1 0 0
        r |= p & (board << (3 * WIDTH)); // 0 1 1 1 => 1 0 0 0
        r |= p & (board >> WIDTH); // 1 0 1 1 => 0 1 0 0
        p >>= 3 * WIDTH; // 1 1 0 => 0 0 1
        r |= p & (board << WIDTH); // 1 1 0 1 => 0 0 1 0
        r |= p & (board >> (3 * WIDTH)); // 1 1 1 0 => 0 0 0 1

        // Positive Diagonal
        let mut p = (board << (WIDTH - 1)) & (board << (2 * (WIDTH - 1)));
        r |= p & (board << (3 * (WIDTH - 1)));
        r |= p & (board >> (WIDTH - 1));
        p >>= 3 * (WIDTH - 1);
        r |= p & (board << (WIDTH - 1));
        r |= p & (board >> (3 * (WIDTH - 1)));

        // Negative Diagonal
        let mut p = (board << (WIDTH + 1)) & (board << (2 * (WIDTH + 1)));
        r |= p & (board << (3 * (WIDTH + 1)));
        r |= p & (board >> (WIDTH + 1));
        p >>= 3 * (WIDTH + 1);
        r |= p & (board << (WIDTH + 1)); // problem
        r |= p & (board >> (3 * (WIDTH + 1)));

        r & (BOARD_MASK ^ mask)
    }

    /// Count of possible wins for the current player
    pub fn curr_win_count(&self) -> u32 {
        (self.possible_mask() & self.curr_win_mask()).count_ones()
    }
    /// Can the current player win on their next move
    pub fn curr_can_win(&self) -> bool {
        self.curr_win_count() > 0
    }

    /// Bitmask of every possible move that doesn't
    /// allow the opponent to win in their next move
    /// Returns `Err(())` if there are none, otherwise
    /// `Ok(mask)`
    fn possible_nonlosing_mask(&self) -> Result<u64, ()> {
        let possible = self.possible_mask();
        let opp_win = self.opp_win_mask();

        let mut mask = possible;

        let forced = possible & opp_win;
        if forced > 0 {
            if forced & (forced - 1) > 0 {
                return Err(()); // Opponent has two possible wins
            } else {
                mask = forced; // We must block the opponent
            }
        }

        mask &= !(opp_win >> 1); // Avoid playing below an opponent win
        if mask == 0 {
            return Err(());
        } // No nonlosing option
        Ok(mask)
    }

    /// Returns an iterator over the resulting board
    /// for every move that doesn't allow the opponent to win
    /// in their next move. Returns `Err(())` if there are none
    /// (i.e. the current player will lose), otherwise `Result(nexts)`
    pub fn possible_nonlosing_nexts(
        &self,
    ) -> Result<impl Iterator<Item = (column::Idx, Self)>, ()> {
        let mask = self.possible_nonlosing_mask()?;
        Ok(column::CENTRED.iter().filter_map(move |&col| {
            match (col_mask(col) & mask) > 0 {
                true => Some((col, self.placed(col, self.curr()).unwrap())),
                false => None,
            }
        }))
    }

    /// Is the board already won?
    fn is_won(board: u64) -> bool {
        // Horizontal
        let m = board & (board >> WIDTH);
        if m & (m >> (2 * WIDTH)) > 0 {
            return true;
        }

        // Negative Diagonal
        let m = board & (board >> (WIDTH - 1));
        if m & (m >> (2 * (WIDTH - 1))) > 0 {
            return true;
        }

        // Positive Diagonal
        let m = board & (board >> (WIDTH + 1));
        if m & (m >> (2 * (WIDTH + 1))) > 0 {
            return true;
        }

        // Vertical alignment
        let m = board & (board >> 1);
        if m & (m >> 2) > 0 {
            return true;
        }

        false
    }

    /// Heuristic to order move exploration
    pub fn heuristic(&self) -> u32 {
        self.curr_win_mask().count_ones()
    }
}

impl Board for BitBoard {
    const EMPTY: Self = BitBoard { board: 0, mask: 0 };

    #[inline(always)]
    fn count_moves(&self) -> usize {
        self.board.count_ones() as usize + (self.board ^ self.mask).count_ones() as usize
    }

    #[inline(always)]
    fn calc_curr(&self) -> Token {
        match self.count_moves() % 2 {
            0 => Token::STARTING,
            1 => Token::SECOND,
            _ => unreachable!(),
        }
    }

    fn get(&self, cell: Cell) -> Option<Token> {
        if self.mask & cell_mask(cell) == 0 {
            return None;
        };

        if self.board & cell_mask(cell) == 0 {
            Some(self.calc_curr().opp())
        } else {
            Some(self.calc_curr())
        }
    }

    fn col_count(&self, col: column::Idx) -> usize {
        (self.mask & col_mask(col)).count_ones() as usize
    }

    fn top(&self, col: column::Idx) -> Option<Cell> {
        match self.col_count(col) {
            0 => None,
            r => Some(Cell {
                col,
                row: row::Idx::raw(r - 1),
            }),
        }
    }

    fn can_place(&self, col: column::Idx) -> bool {
        self.mask
            & cell_mask(Cell {
                col,
                row: row::Idx::TOP,
            })
            == 0
    }

    fn force_place(&mut self, col: column::Idx, token: Token) {
        if token == self.calc_curr() {
            self.board ^= self.mask;
            self.mask |= self.mask + bottom_col_mask(col);
        } else {
            self.mask |= self.mask + bottom_col_mask(col);
            self.board ^= self.mask;
        }
    }

    fn is_won_at(&self, cell: Cell) -> bool {
        match self.get(cell) {
            Some(token) if token == self.curr() => Self::is_won(self.board),
            Some(_) => Self::is_won(self.board ^ self.mask),
            None => false
        }
    }
}

impl CloneBoard for BitBoard {}

impl MutBoard for BitBoard {
    fn unplace(&mut self, col: column::Idx) {
        let bit_mask = cell_mask(self.top(col).unwrap());
        self.mask &= !bit_mask;
        self.board &= !bit_mask;
        self.board ^= self.mask;
    }
}

impl HashBoard for BitBoard {
    #[inline(always)]
    fn key(&self) -> u64 {
        self.board + self.mask + BOTTOM_MASK
    }
}

impl Position for BitBoard {
    #[inline(always)]
    fn move_count(&self) -> usize {
        self.count_moves()
    }
    #[inline(always)]
    fn curr(&self) -> Token {
        self.calc_curr()
    }
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "BitBoard {{ board: {}, mask: {}, curr: {}}}",
            self.board,
            self.mask,
            self.curr()
        )?;
        for row in row::TOP_DOWN {
            for col in column::COLUMNS {
                let b = self.board & cell_mask(Cell { row, col }) != 0;
                write!(f, "{:b}", b as u8)?;
            }
            write!(f, "   ")?;
            for col in column::COLUMNS {
                let b = self.mask & cell_mask(Cell { row, col }) != 0;
                write!(f, "{:b}", b as u8)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Moves;

    make_board_tests!(BitBoard);
    make_mut_board_tests!(BitBoard);

    #[test]
    fn possible_is_possible() {
        for _ in 0..100 {
            for len in 0..40 {
                let moves = Moves::random(len);
                let b = BitBoard::from_moves(&moves);
                let possible = b.possible_mask();
                for col in column::COLUMNS {
                    if col_mask(col) & possible > 0 {
                        assert!(
                            b.clone().place(col, b.curr()).is_some(),
                            "Could not place in col given by possible @ {col}"
                        );
                    }
                }
            }
        }
    }

}
