use std::fmt::Display;
use std::fmt::Formatter;
use std::ops::Not;

use crate::basic::finite_index::FiniteIndex;

mod finite_index;

/// Token on the connect four board.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Token {
    A,
    B,
}

impl Token {
    /// The starting token
    pub const STARTING: Token = Token::A;
    /// The secondary token
    pub const SECOND: Token = Token::B;

    /// Get the next, opposite, other token
    #[inline(always)]
    pub const fn next(&self) -> Token {
        match self {
            Token::A => Token::B,
            Token::B => Token::A,
        }
    }

    /// Get the previous token. (Same as `next()`)
    #[inline(always)]
    pub const fn prev(&self) -> Token {
        self.next()
    }

    /// The opposing token. (Same as `next()`)
    #[inline(always)]
    pub const fn opp(&self) -> Token {
        self.next()
    }
}

impl Not for Token {
    type Output = Token;

    fn not(self) -> Self::Output {
        self.next()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::A => write!(f, "Token(A)"),
            Token::B => write!(f, "Token(B)"),
        }
    }
}

pub mod column {
    use super::*;
    /// Number of columns on a Connect Four board
    pub const COUNT: usize = 7;

    pub type Idx = FiniteIndex<COUNT>;

    /// Column indexes from left to right
    pub const COLUMNS: [Idx; COUNT] = [
        Idx::raw(0),
        Idx::raw(1),
        Idx::raw(2),
        Idx::raw(3),
        Idx::raw(4),
        Idx::raw(5),
        Idx::raw(6),
    ];

    /// Column indexes on the left side of the board, left to right, not including centre
    pub const LEFT_SIDE: [Idx; 3] = [Idx::raw(0), Idx::raw(1), Idx::raw(2)];
    /// Column indexes on the right side of the board, right to left, not including centre
    pub const RIGHT_SIDE: [Idx; 3] = [Idx::raw(6), Idx::raw(5), Idx::raw(4)];

    /// Centred column indexes, starting in the middle and moving outward
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
        /// The leftmost column
        pub const LEFT: Self = Idx::raw(0);
        /// The centre column
        pub const CENTRE: Self = Idx::raw(3);
        /// The rightmost column
        pub const RIGHT: Self = Idx::raw(6);

        /// Is the column strictly on the left side of the board
        #[inline(always)]
        pub fn is_left_side(&self) -> bool {
            usize::from(*self) < 3
        }

        /// Is the column strictly on the right side of the board
        #[inline(always)]
        pub fn is_right_side(&self) -> bool {
            usize::from(*self) > 3
        }

        /// Returns the column on the opposite side of the board, based on vertical symmetry.
        #[inline(always)]
        pub fn mirrored(self) -> Self {
            Self::raw(usize::from(Self::MAX) - usize::from(self))
        }

        /// Returns the 1-indexed digit of the column
        pub fn to_digit(self) -> char {
            char::from_digit(1 + u32::from(self), 10).unwrap()
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

    /// The number of rows on a Connect Four board
    pub const COUNT: usize = 6;
    pub type Idx = FiniteIndex<COUNT>;

    impl Idx {
        /// The bottom row
        pub const BOTTOM: Self = Self::ZERO;
        /// The top row
        pub const TOP: Self = Self::MAX;
    }

    /// Row indexes from bottom to top
    pub const BOTTOM_UP: [Idx; COUNT] = [
        Idx::raw(0),
        Idx::raw(1),
        Idx::raw(2),
        Idx::raw(3),
        Idx::raw(4),
        Idx::raw(5),
    ];
    /// Row indexes from top to bottom
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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
    RightUp,
    LeftDown,
    RightDown,
    LeftUp,
}

impl Dir {
    /// Transform Dir into `(col_offset, row_offset)`
    #[inline(always)]
    pub fn into_dir(self) -> (isize, isize) {
        match self {
            Dir::Up => (0, 1),
            Dir::Down => (0, -1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
            Dir::RightUp => (1, 1),
            Dir::LeftDown => (-1, -1),
            Dir::RightDown => (1, -1),
            Dir::LeftUp => (-1, 1),
        }
    }
}

impl From<Dir> for (isize, isize) {
    fn from(value: Dir) -> Self {
        value.into_dir()
    }
}

impl Cell {
    /// The number of cells on a connect four board.
    pub const COUNT: usize = column::COUNT * row::COUNT;

    /// Tries to shift the cell by (col, row), returning `None`
    /// when out of bounds
    #[inline(always)]
    pub fn try_shift(&self, dir: Dir, by: usize) -> Option<Cell> {
        let by: isize = isize::try_from(by).unwrap();
        let (col_offset, row_offset) = dir.into();
        let col = self.col.try_shift(col_offset * by)?;
        let row = self.row.try_shift(row_offset * by)?;
        Some(Cell { col, row })
    }

    /// The cell above, returning `None` for any cell in the
    /// top row of the board
    #[inline(always)]
    pub fn above(&self) -> Option<Cell> {
        self.try_shift(Dir::Up, 1)
    }

    /// Returns an iterator over cells in the given direction
    /// from the given cell for distances 1 to 3. Does not include the
    /// given cell.
    #[inline(always)]
    pub fn nbhd_in(&self, dir: Dir) -> impl Iterator<Item = (usize, Cell)> {
        (1..=3).map_while(move |by| Some((by, self.try_shift(dir, by)?)))
    }

    /// The cell on the mirrored position on the board, based
    /// on vertical symmetry.
    #[inline(always)]
    pub fn mirrored(&self) -> Self {
        Cell {
            col: self.col.mirrored(),
            row: self.row,
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell({}, {})", self.row, self.col)
    }
}
