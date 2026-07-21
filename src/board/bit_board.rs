use crate::basic::*;
use crate::board::HashBoard;
use crate::board::{Board, CloneBoard, MutBoard, bit_col::BitCol};
use crate::algorithms::Position;

use std::hash::Hash;
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
#[derive(Clone, PartialEq, Eq)]
pub struct BitBoard {
    board: u64,
    mask: u64
}

const WIDTH: u64 = column::COUNT as u64;

const BOTTOM_MASK: u64 = {
    let mut mask: u64 = 0;
    let mut i = 0;
    while i < column::COUNT {
        mask |= 1 << (WIDTH * i as u64);
        i += 1;
    }
    mask
};

const ABOVE_MASK: u64 = BOTTOM_MASK << 6;
const BOARD_MASK: u64 = ((1 << (WIDTH * WIDTH - 1)) - 1) & (!ABOVE_MASK);

const fn row_mask(row: row::Idx) -> u64 {
    BOTTOM_MASK << row.to_u64()
}

const fn bottom_col_mask(col: column::Idx) -> u64 {
    1 << (WIDTH * col.to_u64())
}

const fn col_mask(col: column::Idx) -> u64 {
    0b111111 << (WIDTH * col.to_u64())
}

const fn cell_mask(cell: Cell) -> u64 {
    (1 << cell.row.to_u64()) << (WIDTH * cell.col.to_u64())
}

impl BitBoard {
    fn possible_mask(&self) -> u64 {
        (self.mask + BOTTOM_MASK) & BOARD_MASK
    }

    pub fn placed_curr_unchecked(&self, col: column::Idx) -> Self {
        BitBoard {
            board: self.board ^ self.mask,
            mask: self.mask | (self.mask + bottom_col_mask(col))
        }
    }

    fn curr_win_mask(&self) -> u64 { Self::win_mask(self.board, self.mask) }
    fn opp_win_mask(&self) -> u64 { Self::win_mask(self.board ^ self.mask, self.mask )}

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

    pub fn curr_win_count(&self) -> u32 { (self.possible_mask() & self.curr_win_mask()).count_ones() }
    pub fn curr_can_win(&self) -> bool { self.curr_win_count() > 0 }

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
        if mask == 0 { return Err(()); } // No nonlosing option
        Ok(mask)
    }

    pub fn possible_nonlosing_nexts(&self) -> Result<impl Iterator<Item=(column::Idx, Self)>, ()> {
        let mask = self.possible_nonlosing_mask()?;
        Result::Ok(column::CENTRED.iter()
            .filter_map(move |&col| {
                match ((col_mask(col) & mask) > 0) {
                    true => Some((col, self.placed(col, self.curr()).unwrap())),
                    false => None
                }
            })
        )
    }

    pub fn heuristic(&self) -> u32 {
        self.curr_win_mask().count_ones()
    }
}

impl Board for BitBoard {
    const EMPTY: Self = BitBoard {
        board: 0,
        mask: 0
    };

    fn count_moves(&self) -> usize {
        self.board.count_ones() as usize
        + (self.board ^ self.mask).count_ones() as usize
    }

    fn calc_curr(&self) -> Token {
        match self.count_moves() % 2 {
            0 => Token::STARTING,
            1 => Token::SECOND,
            _ => unreachable!()
        }
    }

    fn get(&self, cell: Cell) -> Option<Token> {
        if self.mask & cell_mask(cell) == 0 { return None };

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
            r => Some(Cell { col, row: row::Idx::raw(r-1) })
        }
    }

    fn can_place(&self, col: column::Idx) -> bool {
        self.mask & cell_mask(Cell { col, row: row::Idx::TOP }) == 0
    }

    fn force_place(&mut self, col: column::Idx, token: Token) {
        if (token == self.calc_curr()) {
            self.board ^= self.mask; 
            self.mask |= self.mask + bottom_col_mask(col);
        } else {
            self.mask |= self.mask + bottom_col_mask(col);
            self.board ^= self.mask; 
        }
    }

    fn is_won(&self, token: Token) -> bool {
        let board = if token == self.calc_curr() {
            self.board
        } else {
            self.board ^ self.mask
        };

        // Horizontal
        let m = board & (board >> WIDTH);
        if m & (m >> (2 * WIDTH)) > 0 { return true; }

        // Negative Diagonal
        let m = board & (board >> (WIDTH-1));
        if m & (m >> (2*(WIDTH - 1))) > 0 { return true; }

        // Positive Diagonal
        let m = board & (board >> (WIDTH+1));
        if m & (m >> (2*(WIDTH+1))) > 0 { return true; }

        // Vertical alignment
        let m = board & (board >> 1);
        if m & (m >> 2) > 0 { return true; }

        false
    }

    fn is_won_at(&self, cell: Cell) -> bool { self.is_won(self.get(cell).unwrap()) }
}

impl CloneBoard for BitBoard { }

impl MutBoard for BitBoard {
    fn unplace(&mut self, cell: Cell) {
        let bit_mask = cell_mask(cell);
        self.mask &= !bit_mask;
        self.board &= !bit_mask;
        self.board ^= self.mask;
    }
}

impl HashBoard for BitBoard {
    fn key(&self) -> u64 {
        self.board + self.mask + BOTTOM_MASK
    }
}

impl Position for BitBoard {
    fn move_count(&self) -> usize { self.count_moves() }
    fn curr(&self) -> Token { self.calc_curr() }
}

fn show_mask(mask: u64) -> String {
    let mut string = String::new();
    for col in column::LEFT..=column::RIGHT {
        let b = mask & ABOVE_MASK & (col_mask(col) << 1) != 0;
        string += &format!("{:b}", b as u8);
    }
    string += "--";
    for row in row::TOP_DOWN {
        string += "\n";
        for col in column::LEFT..=column::RIGHT {
            let b = mask & cell_mask(Cell { row, col }) != 0;
            string += &format!("{:b}", b as u8);
        }
    }
    string += " | ";
    string += &format!("{:b}", mask >> 49);
    string += "\n";

    string
}

impl Debug for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BitBoard {{ board: {}, mask: {}, curr: {}}}", self.board, self.mask, self.curr())?;
        for row in row::TOP_DOWN {
            for col in column::LEFT..=column::RIGHT {
                let b = self.board & cell_mask(Cell { row, col }) != 0;
                write!(f, "{:b}", b as u8)?;
            }
            write!(f, "   ");
            for col in column::LEFT..=column::RIGHT {
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
                for col in column::LEFT..=column::RIGHT {
                    if col_mask(col) & possible > 0 {
                        assert!(b.clone().place(col, b.curr()).is_some(), "Could not place in col given by possible @ {col}");
                    }
                }

            }
        }
    }

    #[test]
    fn nonlosing() {
        for _ in 0..1000 {
            for len in 0..41 {
                let moves = Moves::random(len);
                let b = BitBoard::from_moves(&moves);
                if b.is_won(b.curr()) || b.is_won(b.opp()) { continue; }
                let Ok(mut nexts) = b.possible_nonlosing_nexts() else {
                    assert!(b.nexts(b.curr()).all(|(col, next)| next.curr_can_win()), "nonlosing missed a nonlosing move");
                    continue;
                };
                assert!(nexts.all(|(col, next)| !next.curr_can_win()), "nonlosing returned a losing move");
            }
        }
    }
}
