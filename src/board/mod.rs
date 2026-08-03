use crate::basic::*;
use std::fmt::Debug;

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

    /// Returns the token at the given cell, or None if the cell is empty.
    fn get(&self, cell: Cell) -> Option<Token>;

    /// Compute the current player based on the number of tokens on the board.
    /// Warning: default implementation is slow
    #[inline(always)]
    fn calc_curr(&self) -> Token {
        match self.count_moves() % 2 {
            0 => Token::STARTING,
            1 => Token::SECOND,
            _ => unreachable!(),
        }
    }

    /// Compute the total number of moves played.
    /// Warning: default implementation is slow
    fn count_moves(&self) -> usize {
        let mut count = 0;
        for row in row::BOTTOM_UP {
            for col in column::COLUMNS {
                let cell = Cell { col, row };
                if self.get(cell).is_some() {
                    count += 1
                }

            }
        }
        count
    }

    /// Compute the count of tokens in the given column.
    /// Warning: default implementation is slow
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
    fn can_place(&self, col: column::Idx) -> bool {
        self.col_count(col) < row::COUNT
    }

    /// Returns an iterator over every possible next move.
    /// Ordered by `column::CENTRED`.
    fn next_cells(&self) -> impl Iterator<Item = Cell> {
        column::CENTRED.into_iter().filter_map(|col| {
            let row = row::Idx::try_from(self.col_count(col)).ok()?;
            Some(Cell { row, col })
        })
    }
    /// Force places the given token in the given column.
    /// Must ensure the column is not full before (i.e. by checking `can_place`)
    /// `token` should equal the current player, as given by `curr_player`.
    /// Panics if the column is full.
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
    /// (left, right, down, left-down, right-down) if that token *were* placed
    /// in that cell.
    /// Returns `None` if a possible win is found (a row of four), else
    /// `Some((adjacent_pair_count, adjacent_triple_count))`
    fn count_adjacent_around(&self, cell: Cell, token: Token) -> Option<(usize, usize)> {
        // count of matching tokens in the given direction, *including* the given token.
        let count = |dir| {
            cell.nbhd_in(dir)
                .take_while(|&(_, other)| self.get(other) == Some(token))
                .last()
                .unwrap_or((0, cell))
                .0
                + 1
        };

        let mut pairs = 0;
        let mut triples = 0;
        // Consider the given calculated run count,
        // mutating pairs/triples as necessary and returning
        // None if a win is found.
        let mut consider = |count| match count {
            0 => unreachable!(),
            1 => Some(()),
            2 => { pairs += 1; Some(()) }
            3 => { triples += 1; Some(()) }
            4.. => None,
        };
        // horizontal = left + right - 1
        consider(count(Dir::Left) + count(Dir::Right) - 1)?;
        // vertical = down
        consider(count(Dir::Down))?;
        // diag_pos = left-down + right-up - 1
        consider(count(Dir::LeftDown) + count(Dir::RightUp) - 1)?;
        // diag_neg = right-down + left-up - 1
        consider(count(Dir::RightDown) + count(Dir::LeftUp) - 1)?;
        Some((pairs, triples))
    }

    /// Checks there is a win (a sequence of four same-colour tokens) that includes the token
    /// at the given cell. The winning player is given by the colour of the token at the cell.
    /// Panics if the column is empty.
    fn is_won_at(&self, cell: Cell) -> bool {
        let token = self.get(cell).unwrap();
        self.count_adjacent_around(cell, token).is_none()
    }

    /// Checks if there is a win(sequence fo four same-colour tokens) that includes the token
    /// at the top of the given column.
    /// Panics if the column is empty.
    fn is_won_at_col(&self, col: column::Idx) -> bool {
        let cell = self.top(col).unwrap();
        self.is_won_at(cell)
    }

    /// Pretty display to string
    fn to_display(&self) -> String {
        let mut string = String::new();
        println!("_ 1 2 3 4 5 6 7 _");
        for &row in row::BOTTOM_UP.iter().rev() {
            string.push_str("| ");
            for col in column::COLUMNS {
                let cell = Cell { col, row };
                match self.get(cell) {
                    Some(Token::B) => string.push_str("O "),
                    Some(Token::A) => string.push_str("X "),
                    None => string.push_str("⋅ "),
                }
            }
            string.push_str("|\n");
        }

        string
    }

    /// Pretty prints the board to stdout.
    fn display(&self) {
        println!("{}", self.to_display());
    }

    /// Read a board from a visual string representation.
    fn from_display(string: &str) -> Self {
        let mut board = Self::EMPTY;
        for line in string.split('|').rev() {
            if !line.chars().any(|ch| matches!(ch, 'A' | 'B' | 'X' | 'O' )){
                continue;
            }


            for (ch, col) in line.replace(" ", "").chars().zip(column::COLUMNS) {
                let token = match ch {
                    'A' | 'X' => Token::A,
                    'B' | 'O' => Token::B,
                    '⋅' => continue,
                    '+' | '-' => break,
                    ch => panic!("Invalid character in board string: '{}'", ch),
                };
                board.place(col, token);
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
        board
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
    /// Ordered by `column::CENTRED`.
    fn nexts(&self, token: Token) -> impl Iterator<Item = (column::Idx, Self)> {
        // a simple optimisation to try the centre columns first
        column::CENTRED.iter().filter_map(move |&col| {
            let next = self.placed(col, token)?;
            Some((col, next))
        })
    }

    /// Is there a possible winning move for the given token
    fn can_win(&self, token: Token) -> bool {
        self.nexts(token).any(|(col, board)| board.is_won_at_col(col))
    }

    /// Horizontal reflection by copying each cell to its mirrored position
    /// based on vertical symmetry.
    fn mirrored(&self) -> Self {
        let mut board = Self::EMPTY;
        for col in column::COLUMNS {
            for row in row::BOTTOM_UP {
                if let Some(token) = self.get(Cell { col, row }) {
                    board.place(col.mirrored(), token);
                }
            }
        }
        board
    }

    fn dfs(&self, curr: Token, depth: usize, visit_leaf: &mut impl FnMut(&Self)) {
        if depth == 0 {
            return visit_leaf(self);
        }
        for (_, next) in self.nexts(curr) {
            next.dfs(curr.next(), depth - 1, visit_leaf)
        }
    }
}

/// Trait for board implementations that don't have a cheap clone operation
/// and instead place and unplace tokens on the same board.
pub trait MutBoard: Board {
    /// Removes the token at the given cell, modifying the board in place.
    /// Does not check if there is a token at the cell.
    fn unplace(&mut self, col: column::Idx);
}

/// Trait for board implementations that have a cheap non-zero hash function `key`,
/// used for caching board states in the solver. Must only take up
/// `column::COUNT * column::COUNT = 49` bits.
pub trait HashBoard: Board {
    /// Returns a 49-bit hash key for the board state.
    fn key(&self) -> u64;

    /// Count the number of moves from the key
    fn depth(key: u64, curr: Token) -> usize;
}
