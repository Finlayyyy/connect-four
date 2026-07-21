use crate::{
    board::{Board, HashBoard},
    solver_utils::{Position, position},
};
use hashbrown::HashTable;

/// Very large prime number for hash table size
/// * sizeof::<u64>() = ~200MB
pub const LARGE_SIZE: u64 = 25_165_843;

// Relatively small prime number for smaller hash table
// * sizeof::<u64>() = ~1MB
pub const SMALL_SIZE: u64 = 196613;

pub struct HashMap {
    table: Vec<u64>,
    max_count: u64,
}

impl HashMap {
    const KEY_MASK: u64 = (1 << 55) - 1;
    const KEY_OFFSET: u64 = 1;
    const SCORE_MASK: u64 = (1 << 8) - 1;
    const SCORE_OFFSET: u64 = 56;

    const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

    pub fn new(max_count: u64) -> Self {
        HashMap {
            table: vec![0; max_count.try_into().unwrap()],
            max_count,
        }
    }

    fn hash(&self, key: u64) -> usize {
        (key % self.max_count) as usize
    }

    fn pack_entry(key: u64, score: isize) -> u64 {
        let score = u64::try_from(score - position::MIN_SCORE).unwrap();
        1 | (key << Self::KEY_OFFSET) | (score << Self::SCORE_OFFSET)
    }

    /// MIN_SCORE <= score <= MAX_SCORE
    pub fn insert<B: HashBoard + Position>(&mut self, board: &B, score: isize) {
        // Ensure key fits in 55 bits
        debug_assert!(board.key() & (!Self::KEY_MASK) == 0);
        if board.move_count() > Self::MAX_CACHE_DEPTH {
            return;
        }

        let key = board.key();
        let entry = Self::pack_entry(key, score);

        let hash = self.hash(key);
        self.table[hash] = entry;
    }

    fn unpack_entry(entry: u64) -> Option<(u64, isize)> {
        if entry & 1 == 0 {
            return None;
        }
        let key = (entry >> Self::KEY_OFFSET) & Self::KEY_MASK;
        let score = (entry >> Self::SCORE_OFFSET) & Self::SCORE_MASK;
        let score = isize::try_from(score).unwrap() + position::MIN_SCORE;
        Some((key, score))
    }

    pub fn get<B: HashBoard>(&mut self, board: &B) -> Option<isize> {
        let key = board.key();
        let hash = self.hash(key);
        let entry = self.table[hash];
        let (other_key, score) = Self::unpack_entry(entry)?;
        match key == other_key {
            true => Some(score),
            false => None,
        }
    }

    pub fn allocation_size(&self) -> usize {
        size_of::<u64>() * usize::try_from(self.max_count).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic::*;
    use crate::benching::{END_EASY, read_testset};
    use crate::board::*;

    #[test]
    fn pack_unpack() {
        let testset = read_testset(END_EASY);
        for (moves, score) in testset {
            let board = BitCols::from_moves(&moves);
            let key = board.key();
            let entry = HashMap::pack_entry(key, score);
            let (key_2, score_2) = HashMap::unpack_entry(entry).unwrap();
            assert_eq!(key, key_2, "(unpack∘pack)(key) != key for {key}");
            assert_eq!(score, score_2, "(unpack∘pack)(score) != score for {score}");
        }
    }
}
