use crate::basic::*;
use crate::board::HashBoard;
use crate::board::{Board, CloneBoard, MutBoard, bit_col::BitCol};
use crate::solver_utils::Position;

/// A board implementation using bit manipulation for storage.
/// Each column is stored as a BitCol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitCols {
    cols: [BitCol; column::COUNT],
}

impl BitCols {
    pub fn get_col(&self, col: column::Idx) -> BitCol {
        self.cols[usize::from(col)]
    }

    pub fn get_cols(&self) -> &[BitCol] {
        &self.cols
    }
}

impl Board for BitCols {
    const EMPTY: Self = BitCols {
        cols: [BitCol::EMPTY; column::COUNT],
    };

    fn get(&self, cell: Cell) -> Option<Token> {
        self.get_col(cell.col).get(cell.row)
    }

    fn count_moves(&self) -> usize {
        self.cols.iter().map(|col| col.count()).sum()
    }

    fn col_count(&self, col: column::Idx) -> usize {
        self.get_col(col).count()
    }

    fn can_place(&self, col: column::Idx) -> bool {
        !self.get_col(col).is_full()
    }

    fn force_place(&mut self, col: column::Idx, token: Token) {
        let col_idx = usize::from(col);
        self.cols[col_idx].force_push(token);
    }

    fn place(&mut self, col: column::Idx, token: Token) -> Option<Cell> {
        let col_idx = usize::from(col);
        self.cols[col_idx] = self.cols[col_idx].pushed(token)?;
        Some(Cell {
            col,
            row: self.get_col(col).top().unwrap(), // todo! Should unwrap or no?
        })
    }
}

impl CloneBoard for BitCols {}

impl MutBoard for BitCols {
    fn unplace(&mut self, col: column::Idx) {
        self.cols[usize::from(col)].force_pop();
    }
}

impl HashBoard for BitCols {
    fn key(&self) -> u64 {
        let mut k = 0;
        for (i, col) in self.cols.iter().enumerate() {
            k |= col.to_u64() << (i * 8);
        }
        k
    }
}

impl Position for BitCols {
    fn move_count(&self) -> usize {
        self.count_moves()
    }
    fn curr(&self) -> Token {
        self.calc_curr()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    make_board_tests!(BitCols);
    make_mut_board_tests!(BitCols);
}

#[cfg(test)]
mod pos_tests {
    use super::*;
    use crate::solver_utils::*;
    make_board_tests!(WithInfo<BitCols>);
    make_mut_board_tests!(WithInfo<BitCols>);
}
