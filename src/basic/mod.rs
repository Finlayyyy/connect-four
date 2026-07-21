use std::cmp::min;
use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Not;

use crate::basic::finite_index::FiniteIndex;

mod finite_index;

/// Token on the board. A starts.
/// May be represented as Red and Yellow
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Token {
    A,
    B,
}

impl Token {
    /// The starting token 
    pub const STARTING: Token = Token::A;
    pub const SECOND: Token = Token::B;

    /// Get the next, opposite, other token
    pub const fn next(&self) -> Token {
        match self {
            Token::A => Token::B,
            Token::B => Token::A,
        }
    }

    /// next()
    pub const fn prev(&self) -> Token { self.next() }
    pub const fn opp(&self) -> Token { self.next() }
}

impl Not for Token {
    type Output = Token;

    fn not(self) -> Self::Output { self.next() }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::A => write!(f, "Token A"),
            Token::B => write!(f, "Token B"),
        }
    }
}

pub mod column {
    use super::*;

    pub const COUNT: usize = 7;
    pub type Idx = FiniteIndex<COUNT>;

    pub const LEFT: Idx = Idx::LEFT;
    pub const CENTRE: Idx = Idx::CENTRE;
    pub const RIGHT: Idx = Idx::RIGHT;

    pub const CENTRED: [Idx; COUNT] = [
        Idx::raw(3),
        Idx::raw(2),
        Idx::raw(4),
        Idx::raw(1),
        Idx::raw(5),
        Idx::raw(0),
        Idx::raw(6),
    ];

    impl Idx {
        pub const LEFT: Self = Idx::raw(0);
        pub const CENTRE: Self = Idx::raw(3);
        pub const RIGHT: Self = Idx::raw(6);

        pub fn is_left(&self) -> bool {
            usize::from(*self) < 3
        }

        pub fn is_centre(&self) -> bool {
            *self == CENTRE
        }

        pub fn is_right(&self) -> bool {
            usize::from(*self) < 3
        }

        /// Returns the column on the opposite side of the board, based on symmetry.
        pub fn reflected(self) -> Self {
            Self::raw(usize::from(Self::MAX) - usize::from(self))
        }
    }

    impl Display for Idx {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "col({})", usize::from(*self) + 1)
        }
    }
}

pub mod row {
    use super::*;

    pub const COUNT: usize = 6;

    pub type Idx = FiniteIndex<COUNT>;

    impl Idx {
        pub const BOTTOM: Self = Self::ZERO;
        pub const TOP: Self = Self::MAX;
    }

    /// bottom to top
    pub const BOTTOM_UP: [Idx; COUNT] = [
        Idx::raw(0),
        Idx::raw(1),
        Idx::raw(2),
        Idx::raw(3),
        Idx::raw(4),
        Idx::raw(5),
    ];

    pub const TOP_DOWN: [Idx; COUNT] = [
        Idx::raw(5),
        Idx::raw(4),
        Idx::raw(3),
        Idx::raw(2),
        Idx::raw(1),
        Idx::raw(0),
    ];

    impl Display for Idx {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "row({})", usize::from(*self) + 1)
        }
    }
}

/// A Cell on the board, defined by a column and row index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub col: column::Idx,
    pub row: row::Idx,
}

impl Cell {
    /// Tries to shift by (col, row)
    pub fn try_shift(&self, by: (isize, isize)) -> Option<Cell> {
        let col = self.col.try_shift(by.0)?;
        let row = self.row.try_shift(by.1)?;
        Some(Cell { col, row })
    }

    pub fn above(&self) -> Option<Cell> {
        self.try_shift((0, 1))
    }

    /// Returns an iterator over the cells in the same row as this cell,
    /// from 3 columns to the left to 3 columns to the right (capped at the board edges).
    pub fn row_neighbourhood(&self) -> impl Iterator<Item = Cell> {
        (self.col.shift(-3)..=self.col.shift(3)).map(move |col| Cell { col, row: self.row })
    }

    /// Returns an iterator over the cells in the same column as this cell,
    /// from 3 rows below to 3 rows above (capped at the board edges).
    pub fn col_neighbourhood(&self) -> impl Iterator<Item = Cell> {
        (self.row.shift(-3)..=self.row.shift(3)).map(move |row| Cell { col: self.col, row })
    }

    /// Returns an iterator over the cells in the same diagonal (bottom-left to top-right)
    /// as this cell within a distance of 3 (capped at the board edges).
    pub fn diag1_neighbourhood(&self) -> impl Iterator<Item = Cell> {
        let start_offset = -min(
            3,
            min(
                isize::from(self.col),
                isize::from(self.row),
            ),
        );

        let end_offset = min(
            3,
            min(
                isize::from(column::Idx::MAX) - isize::from(self.col),
                isize::from(row::Idx::MAX) - isize::from(self.row),
            ),
        );

        (start_offset..=end_offset).map(move |offset| Cell {
            col: self.col.shift(offset),
            row: self.row.shift(offset),
        })
    }

    /// Returns an iterator over the cells in the same diagonal (top-left to bottom-right)
    /// as this cell within a distance of 3 (capped at the board edges).
    pub fn diag2_neighbourhood(&self) -> impl Iterator<Item = Cell> {
        let start_offset = -min(
            3,
            min(
                isize::try_from(self.col).unwrap(),
                isize::try_from(row::Idx::MAX).unwrap() - isize::try_from(self.row).unwrap(),
            ),
        );

        let end_offset = min(
            3,
            min(
                isize::try_from(column::Idx::MAX).unwrap() - isize::try_from(self.col).unwrap(),
                isize::try_from(self.row).unwrap(),
            ),
        );

        (start_offset..=end_offset).map(move |offset| Cell {
            col: self.col.shift(offset),
            row: self.row.shift(-offset),
        })
    }

    pub fn reflected(&self) -> Self {
        Cell {
            col: self.col.reflected(),
            row: self.row,
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell({}, {})", self.row, self.col)
    }
}