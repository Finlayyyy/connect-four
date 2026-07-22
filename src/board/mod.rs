use crate::basic::*;

use rand::RngExt;
use std::{
    fmt::{Debug, Display},
    num::IntErrorKind::Empty,
};

#[macro_use]
pub mod testing;
pub mod array_board;
pub mod bit_board;
mod bit_col;
pub mod bit_cols;
pub mod moves;
pub mod symm_board;

pub use array_board::ArrayBoard;
pub use bit_board::BitBoard;
pub use bit_cols::BitCols;
pub use moves::Moves;
pub use symm_board::SymmBoard;

/// Trait containing common board functionality.
pub trait Board: Debug + Sized + Eq {
    /// An empty starting board.
    const EMPTY: Self;

    fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    /// Returns the token at the given cell, or None if the cell is empty.
    fn get(&self, cell: Cell) -> Option<Token>;

    /// Compute the current player based on the number of tokens on the board.
    fn calc_curr(&self) -> Token {
        match self.count_moves() % 2 {
            0 => Token::STARTING,
            1 => Token::SECOND,
            _ => unreachable!(),
        }
    }

    /// Compute the total number of moves played.
    fn count_moves(&self) -> usize {
        let mut count = 0;
        for row in row::BOTTOM_UP {
            for col in column::LEFT..=column::RIGHT {
                let cell = Cell { col, row };
                match self.get(cell) {
                    Some(_) => count += 1,
                    None => {}
                }
            }
        }
        count
    }

    /// Compute the count of tokens in the given column
    fn col_count(&self, col: column::Idx) -> usize {
        let mut height = 0;
        for row in row::BOTTOM_UP {
            if self.get(Cell { col, row }).is_some() {
                height += 1;
            }
        }
        height
    }

    /// Compute the cell at the top of column.
    /// returns `None` if the column is empty
    fn top(&self, col: column::Idx) -> Option<Cell> {
        let row_idx = self.col_count(col).checked_sub(1)?;
        let row = row::Idx::try_from(row_idx).unwrap();
        Some(Cell { row, col })
    }

    /// Returns true if a token can be placed in the given column.
    /// i.e. the column is not full.
    fn can_place(&self, col: column::Idx) -> bool;

    fn next_cells(&self) -> impl Iterator<Item = Cell> {
        column::CENTRED.into_iter().filter_map(|col| {
            let row = row::Idx::try_from(self.col_count(col)).ok()?;
            Some(Cell { row, col })
        })
    }
    /// Force places the given token in the given column.
    /// Must ensure the column is not full before (i.e. by checking `can_place`)
    /// `token` should equal the current player, as given by `curr_player`.
    fn force_place(&mut self, col: column::Idx, token: Token) {
        debug_assert!(self.can_place(col));
        self.place(col, token).unwrap();
    }

    /// Tries to place the given token in the given column.
    /// Returns `Some(Cell)` if successful, `None` if the column is full.
    /// `token` should equal the current player, as given by `curr_player`.
    fn place(&mut self, col: column::Idx, token: Token) -> Option<Cell> {
        match self.can_place(col) {
            true => {
                self.force_place(col, token);
                self.top(col)
            }
            false => None,
        }
    }

    /// For a given column and token,
    /// calculate the length of same-colour tokens in each (relevant) direction
    /// (left, right, down, left-down, right-down) if that token *were* placed in that column.
    /// Returns None if a win is found (a row of four), else
    /// Some((count of adjacent pairs, count of adjacent triples))
    fn count_adjacent_at(&self, cell: Cell, token: Token) -> Option<(usize, usize)> {
        let count_line = |dir: (isize, isize)| {
            for d in 1..=3 {
                match cell.try_shift((d * dir.0, d * dir.1)) {
                    Some(next) if self.get(next) == Some(token) => continue,
                    _ => return d as usize,
                }
            }
            return 4;
        };

        let mut pairs = 0;
        let mut triples = 0;
        let mut consider_count = |count| match count {
            0 => unreachable!(),
            1 => Some(()),
            2 => {
                pairs += 1;
                Some(())
            }
            3 => {
                triples += 1;
                Some(())
            }
            4.. => None,
        };
        // horizontal = left + right
        consider_count(count_line((-1, 0)) + count_line((1, 0)) - 1)?;
        // vertical = down
        consider_count(count_line((0, -1)))?;
        // diag_pos = left-down + right-up
        consider_count(count_line((-1, -1)) + count_line((1, 1)) - 1)?;
        // diag_neg = right-down + left-up
        consider_count(count_line((1, -1)) + count_line((-1, 1)) - 1)?;
        Some((pairs, triples))
    }

    /// Checks there is a win (a sequence of four same-colour tokens) that includes the token
    /// at the given Cell. The winning player is given by the colour of the token at the cell.
    /// Panics if the column is empty.
    fn is_won_at(&self, cell: Cell) -> bool {
        let token = self.get(cell).unwrap();

        self.count_adjacent_at(cell, token).is_none()
    }

    fn is_won_at_col(&self, col: column::Idx) -> bool {
        let cell = self.top(col).unwrap();
        self.is_won_at(cell)
    }

    fn is_won(&self, token: Token) -> bool {
        for col in column::LEFT..=column::RIGHT {
            let Some(top) = self.top(col) else { continue };
            for row in (row::Idx::BOTTOM..=top.row) {
                let cell = Cell { row, col };
                if self.get(cell).unwrap() == token && self.is_won_at(cell) {
                    return true;
                }
            }
        }
        false
    }

    /// String pretty display
    fn to_display(&self) -> String {
        let mut string = String::new();

        for &row in row::BOTTOM_UP.iter().rev() {
            string.push('|');
            for col in column::LEFT..=column::RIGHT {
                let cell = Cell { col, row };
                match self.get(cell) {
                    Some(Token::B) => string.push('O'),
                    Some(Token::A) => string.push('X'),
                    None => string.push('.'),
                }
            }
            string.push_str("|\n");
        }

        string
    }

    /// Pretty prints the board to stdout.
    fn display(&self) {
        print!("{}", self.to_display());
    }

    /// Read a board from a visual string representation.
    /// e.g.
    /// |...R...|
    /// |...Y...|
    /// |...R...|
    /// |...Y...|
    /// |...R...|
    /// |.RYYRY.|
    fn from_display(string: &str) -> Self {
        let mut board = Self::EMPTY;
        for line in string.split('|').rev() {
            if line.trim().is_empty() {
                continue;
            }
            debug_assert_eq!(line.len(), 7);

            for (i, ch) in line.chars().enumerate() {
                let token = match ch {
                    'Y' => Token::A,
                    'X' => Token::A,
                    'R' => Token::B,
                    'O' => Token::B,
                    '.' | ' ' => continue,
                    '+' | '-' => return board, // end of board representation
                    _ => panic!("Invalid character in board string: {}", ch),
                };
                let cell = board.place(column::Idx::try_from(i).unwrap(), token);
                debug_assert!(cell.is_some());
            }
        }

        board
    }

    /// From a sequence of moves
    fn from_moves(moves: &Moves) -> Self {
        let mut board: Self = Board::EMPTY;
        for &(col, token) in moves.iter() {
            board.place(col, token);
        }
        return board;
    }
}

/// Trait for board implementations that have a cheap clone operation.
/// Must opt-in to this trait.
pub trait CloneBoard: Board + Clone {
    /// Clones the given board and calls `try_place` with the given column,
    /// returning the new board and cell if successful.
    /// `token` should equal the current player, as given by `curr_player`.
    fn placed(&self, col: column::Idx, token: Token) -> Option<Self> {
        // check if we can place first to avoid cloning unnecessarily
        match self.can_place(col) {
            false => None,
            true => {
                let mut new_board = self.clone();
                new_board.force_place(col, token);
                Some(new_board)
            }
        }
    }

    /// Returns an iterator over every possible subsequent board state
    /// after placing the given token in each non-full column.
    /// Ordered by column::INDEXES_CENTRED.
    fn nexts(&self, token: Token) -> impl Iterator<Item = (column::Idx, Self)> {
        // a simple optimisation to try the centre columns first
        column::CENTRED.iter().filter_map(move |&col| {
            let next = self.placed(col, token)?;
            Some((col, next))
        })
    }

    fn can_win(&self, token: Token) -> bool {
        self.nexts(token).any(|(col, board)| board.is_won(token))
    }

    // Horizontal reflection by copying each cell to its reflected position
    fn reflected(&self) -> Self {
        let mut board = Self::EMPTY;
        for col in column::LEFT..=column::RIGHT {
            for row in row::BOTTOM_UP {
                if let Some(token) = self.get(Cell { col, row }) {
                    board.place(col.reflected(), token);
                }
            }
        }

        board
    }
}

/// Trait for board implementations that don't have a cheap clone operation
/// and instead place and unplace tokens on the same board.
pub trait MutBoard: Board {
    /// Removes the token at the given cell, modifying the board in place.
    /// Does not check if there is a token at the cell.
    fn unplace(&mut self, col: column::Idx);

    /// To a sequence of moves
    fn to_moves(self) -> Moves {
        todo!();
        let mut board = self;
        let mut moves = vec![];
        let mut prev = board.calc_curr().prev();
        'outer: while !board.is_empty() {
            for col in column::LEFT..=column::RIGHT {
                let Some(cell) = board.top(col) else { continue };
                if board.get(cell).unwrap() == prev {
                    moves.push((col, prev));
                    board.unplace(col);
                    prev = prev.prev();
                    continue 'outer;
                }
            }
            println!("{}", prev);
            board.display();
            panic!("Invalid board to represent as alternating moves");
        }
        moves.reverse();
        Moves { moves }
    }
}

pub trait HashBoard: Board {
    fn key(&self) -> u64;
}
