use std::fmt::Display;

use crate::basic::*;
use crate::board::{Board, MutBoard};

/// Moves implementation using a vector of placed tokens.
/// Stores only the moves made, reconstructing the board state as needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Moves {
    pub moves: Vec<(column::Idx, Token)>,
}

impl Moves {
    /// Returns an iterator of moves made
    pub fn iter(&self) -> std::slice::Iter<'_, (column::Idx, Token)> {
        self.moves.iter()
    }

    /// Read from a moves string
    pub fn from_string(string: &str) -> Self {
        let tokens = std::iter::successors(Some(Token::STARTING), |t| Some(t.next()));
        let cols = string
            .chars()
            .map(|c| column::Idx::from_digit(c).unwrap());
        Moves {
            moves: cols.zip(tokens).collect(),
        }
    }
}

impl Board for Moves {
    const EMPTY: Self = Moves { moves: Vec::new() };

    fn count_moves(&self) -> usize {
        self.moves.len()
    }

    fn get(&self, cell: Cell) -> Option<Token> {
        let mut col_count = 0;
        for (col, token) in &self.moves {
            if *col == cell.col {
                if col_count == usize::from(cell.row) {
                    return Some(*token);
                }
                col_count += 1;
            }
        }
        None
    }

    fn can_place(&self, col: column::Idx) -> bool {
        self.col_count(col) < row::COUNT
    }

    fn place(&mut self, col: column::Idx, token: Token) -> Option<Cell> {
        let row = self.col_count(col);
        let row = row::Idx::try_from(row).ok()?;
        self.moves.push((col, token));
        Some(Cell {
            col,
            row: row::Idx::try_from(row).unwrap(),
        })
    }

    fn col_count(&self, col: column::Idx) -> usize {
        self.moves.iter().filter(|(c, _)| *c == col).count()
    }
}

impl MutBoard for Moves {
    fn unplace(&mut self, col: column::Idx) {
        let idx = self.iter().enumerate().rev().find(|(_, (c, _))| *c == col);
        match idx {
            None => (),
            Some((i, _)) => {
                self.moves.remove(i);
            }
        }
    }
}

impl Display for Moves {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.moves
            .iter()
            .try_for_each(|(col, _)| write!(f, "{}", col.to_digit()))
    }
}

impl IntoIterator for Moves {
    type Item = (column::Idx, Token);

    type IntoIter = <Vec<(column::Idx, Token)> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.moves.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    make_board_tests!(Moves);
    make_mut_board_tests!(Moves);
}

#[cfg(test)]
mod pos_tests {
    use super::*;
    use crate::solver_utils::*;

    make_board_tests!(WithInfo<Moves>);
    make_mut_board_tests!(WithInfo<Moves>);
}
