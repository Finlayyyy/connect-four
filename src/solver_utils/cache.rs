use crate::board::{Board, HashBoard};
use crate::solver_utils::{Position, position};
use hashbrown::HashTable;

pub enum EntryKind {
    Lower,
    Upper,
    Exact,
}

impl EntryKind {
    pub const EMPTY: u64 = 0;
    pub const fn pack(self) -> u64 {
        match self {
            EntryKind::Lower => 0b01,
            EntryKind::Upper => 0b10,
            EntryKind::Exact => 0b11,
        }
    }

    pub const fn unpack(entry: u64) -> Option<Self> {
        match entry {
            0b00 => None,
            0b01 => Some(EntryKind::Lower),
            0b10 => Some(EntryKind::Upper),
            0b11 => Some(EntryKind::Exact),
            _ => panic!("Invalid packed EntryKind"),
        }
    }
}

#[derive(Clone, Copy)]
struct Entry(u64);

impl Entry {
    const KIND_BITS: u64 = 2;
    const KEY_BITS: u64 = 54;
    const SCORE_BITS: u64 = 8;

    const KIND_MASK: u64 = (1 << Self::KIND_BITS) - 1;
    const KEY_MASK: u64 = (1 << Self::KEY_BITS) - 1;
    const SCORE_MASK: u64 = (1 << Self::SCORE_BITS) - 1;

    const KEY_OFFSET: u64 = Self::KIND_BITS;
    const SCORE_OFFSET: u64 = Self::KEY_BITS + Self::KIND_BITS;

    pub const EMPTY: Self = Entry(EntryKind::EMPTY);

    pub fn pack(kind: EntryKind, key: u64, score: isize) -> Self {
        let kind = kind.pack();
        debug_assert!(kind & (!Self::KIND_MASK) == 0);
        debug_assert!(key & (!Self::KEY_MASK) == 0);
        let score = u64::try_from(score - position::MIN_SCORE).unwrap();
        debug_assert!(score & (!Self::SCORE_MASK) == 0);

        Entry(kind | (key << Self::KEY_OFFSET) | (score << Self::SCORE_OFFSET))
    }

    pub fn unpack(&self) -> Option<(EntryKind, u64, isize)> {
        let mut entry = self.0;
        let kind = EntryKind::unpack(entry & Self::KIND_MASK)?;
        let key = (entry >> Self::KEY_OFFSET) & Self::KEY_MASK;
        let score = (entry >> Self::SCORE_OFFSET) & Self::SCORE_MASK;
        let score = isize::try_from(score).unwrap() + position::MIN_SCORE;
        Some((kind, key, score))
    }
}

pub struct Cache {
    table: Vec<Entry>,
    max_count: u64,
}

impl Cache {
    /// Very large prime number for hash table size
    /// * sizeof::<u64>() = ~200MB
    pub const LARGE_SIZE: usize = 25_165_843;

    // Relatively small prime number for smaller hash table
    // * sizeof::<u64>() = ~1MB
    pub const SMALL_SIZE: usize = 196613;

    const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

    pub fn new(max_count: usize) -> Self {
        Cache {
            table: vec![Entry::EMPTY; max_count],
            max_count: max_count as u64,
        }
    }

    fn hash(&self, key: u64) -> usize {
        (key % self.max_count) as usize
    }

    /// MIN_SCORE <= score <= MAX_SCORE
    pub fn insert<B: HashBoard + Position>(&mut self, kind: EntryKind, board: &B, score: isize) {
        if board.move_count() > Self::MAX_CACHE_DEPTH {
            return;
        }

        let key = board.key();
        let entry = Entry::pack(kind, key, score);

        let hash = self.hash(key);
        self.table[hash] = entry;
        // CHECK WEROWSF
    }

    pub fn get<B: HashBoard>(&self, board: &B) -> Option<(EntryKind, isize)> {
        let key = board.key();
        let hash = self.hash(key);
        let entry = self.table[hash];
        let (kind, other_key, score) = entry.unpack()?;
        match key == other_key {
            true => Some((kind, score)),
            false => None,
        }
    }

    pub fn check<B: HashBoard>(&self, board: &B, alpha: isize, beta: isize) -> (isize, isize) {
        match self.get(board) {
            Some(((EntryKind::Lower), score)) if score > alpha => (score, beta),
            Some(((EntryKind::Upper), score)) if score < beta => (alpha, score),
            Some((EntryKind::Exact, score)) => (score, score),
            _ => (alpha, beta)
        }
    }

    pub fn check_insert<B: HashBoard + Position>(
        &mut self,
        board: &B,
        prev_alpha: isize,
        best: isize,
        beta: isize,
    ) {
        let kind = if best <= prev_alpha {
            EntryKind::Upper
        } else if best >= beta {
            EntryKind::Lower
        } else {
            EntryKind::Exact
        };
        self.insert(kind, board, best);
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
            let kind = EntryKind::Exact;
            let board = BitCols::from_moves(&moves);
            let key = board.key();
            let entry = Entry::pack(kind, key, score);
            let (kind_2, key_2, score_2) = entry.unpack().unwrap();
            assert_eq!(key, key_2, "(unpack∘pack)(key) != key for {key}");
            assert_eq!(score, score_2, "(unpack∘pack)(score) != score for {score}");
        }
    }
}
