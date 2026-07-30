use crate::basic::*;

/// Convert a token to a single bit
#[inline(always)]
fn token_to_u8(token: Token) -> u8 {
    match token {
        Token::A => 0,
        Token::B => 1,
    }
}

/// Convert a single bit into a token
#[inline(always)]
fn u8_to_token(bit: u8) -> Token {
    match bit {
        0 => Token::A,
        1 => Token::B,
        a => panic!("Expected either 0 or 1 to convert into token, found {a}")
    }
}

/// A column of the BitBoard, stored as a u8.
/// Formatted with a leading 1 bit, followed by the rows from bottom to top,
/// The top tile is the LSB and the bottom tile is the MSB after the leading 1.
/// Examples:
/// `0b01ab_cdef` : col is full, a at bottom, f at top
/// `0b0000_0001` : col is empty
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitCol(u8);

impl BitCol {
    /// An empty BitCol.
    pub const EMPTY: Self = BitCol(0b0000_0001);

    /// Counts the number of tokens in the column.
    /// Also the bit index of the leading one.
    #[inline(always)]
    pub const fn count(&self) -> usize {
        7 - self.0.leading_zeros() as usize
    }

    /// Is the column empty?
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0b0000_0001
    }

    /// Is the column full i.e. can no more
    /// tokens be pushed onto it
    #[inline(always)]
    pub const fn is_full(&self) -> bool {
        self.0 & 0b1100_0000 != 0
    }

    /// Gets the token at the given row in the column.
    pub fn get(&self, row: row::Idx) -> Option<Token> {
        if self.count() > usize::from(row) {
            // The bit index of the desired row
            let bit_idx = self.count() - usize::from(row) - 1;
            Some(u8_to_token((self.0 >> bit_idx) & 1))
        } else {
            None // The cell is empty
        }
    }

    /// Pops the top token from the column without checking.
    /// Debug asserts that the column is not empty.
    #[inline(always)]
    pub fn force_pop(&mut self) {
        debug_assert!(!self.is_empty(), "Tried to pop from an empty column.");
        self.0 >>= 1;
    }

    /// Pushes a token onto the column without checking.
    /// Debug asserts that the column is not full.
    #[inline(always)]
    pub fn force_push(&mut self, token: Token) {
        debug_assert!(!self.is_full(), "Tried to push onto a full column.");

        let token_bit = token_to_u8(token);
        self.0 <<= 1;
        self.0 |= token_bit;
    }

    /// Returns a new column with the pushed token,
    /// or `None` if the column was full
    #[inline(always)]
    pub fn pushed(self, token: Token) -> Option<BitCol> {
        let token_bit = token_to_u8(token);
        let pushed = (self.0 << 1) | token_bit;

        // check whether pushed is still valid
        if pushed & 0b1000_0000 != 0 {
            None // column has overflowed
        } else {
            Some(BitCol(pushed))
        }
    }

    /// The row of the highest element in the column.
    /// `None` if the column is empty.
    #[inline(always)]
    pub fn top(&self) -> Option<row::Idx> {
        row::Idx::try_from(self.count() as isize - 1).ok()
    }

    /// Returns the underlying u8 value of the column.
    #[inline(always)]
    pub fn to_u8(self) -> u8 {
        self.0
    }

    /// Returns the underlying u8 value of the column as u64
    #[inline(always)]
    pub fn to_u64(self) -> u64 {
        self.0 as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top() {
        let mut col = BitCol::EMPTY;

        assert!(col.is_empty());
        assert_eq!(col.top(), None);

        for row in row::BOTTOM_UP {
            col.force_push(Token::A);
            assert_eq!(col.top(), Some(row));
        }
        assert!(col.is_full());
    }
}
