use crate::board::{Board, HashBoard};
use crate::solver_utils::{Position, position};
use hashbrown::HashTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry(u64);

impl Entry {
    const KIND_BITS: u64 = 2;
    const KEY_BITS: u64 = 49;
    const SCORE_BITS: u64 = 6;
    const DEPTH_BITS: u64 = 6;

    const KIND_MASK: u64 = (1 << Self::KIND_BITS) - 1;
    const KEY_MASK: u64 = (1 << Self::KEY_BITS) - 1;
    const SCORE_MASK: u64 = (1 << Self::SCORE_BITS) - 1;
    const DEPTH_MASK: u64 = (1 << Self::DEPTH_BITS) - 1;

    const KEY_OFFSET: u64 = Self::KIND_BITS;
    const SCORE_OFFSET: u64 = Self::KEY_BITS + Self::KEY_OFFSET;
    const DEPTH_OFFSET: u64 = Self::SCORE_BITS + Self::SCORE_OFFSET;

    pub const EMPTY: Self = Entry(EntryKind::EMPTY);

    pub fn pack(kind: EntryKind, key: u64, score: isize, depth: usize) -> Self {
        let kind = kind.pack();
        debug_assert!(kind & (!Self::KIND_MASK) == 0);
        debug_assert!(key & (!Self::KEY_MASK) == 0);
        let score = u64::try_from(score - position::MIN_SCORE).unwrap();
        debug_assert!(score & (!Self::SCORE_MASK) == 0);
        let depth = u64::try_from(depth).unwrap();
        debug_assert!(depth & (!Self::DEPTH_MASK) == 0);

        Entry(kind | (key << Self::KEY_OFFSET) | (score << Self::SCORE_OFFSET) | (depth << Self::DEPTH_OFFSET))
    }

    pub fn unpack(&self) -> Option<(EntryKind, u64, isize, usize)> {
        let mut entry = self.0;
        let kind = EntryKind::unpack(entry & Self::KIND_MASK)?;
        let key = (entry >> Self::KEY_OFFSET) & Self::KEY_MASK;
        let score = (entry >> Self::SCORE_OFFSET) & Self::SCORE_MASK;
        let score = isize::try_from(score).unwrap() + position::MIN_SCORE;
        let depth = (entry >> Self::DEPTH_OFFSET) & Self::DEPTH_MASK;
        let depth = usize::try_from(depth).unwrap();
        Some((kind, key, score, depth))
    }

    pub fn is_superior_to(&self, other: Entry) -> bool {
        let Some((kind, key, score, depth)) = self.unpack() else {
            return false; // An empty entry is not superior
        };
        let Some((other_kind, other_key, other_score, other_depth)) = other.unpack() else {
            return true; // Non-empty is superior to empty
        };

        if key == other_key { true }
        else { (depth < other_depth + 4) }
    }
}

pub struct Cache {
    table: Vec<Entry>,
    max_count: u64,
}

impl Cache {
    /// Very large prime number for hash table size
    /// * sizeof::<u64>() = ~400MB
    pub const LARGE_SIZE: usize = 50331653;

    // Relatively small prime number for smaller hash table
    // * sizeof::<u64>() = ~2MB
    pub const SMALL_SIZE: usize = 393241;

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
        if board.move_count() > Self::MAX_CACHE_DEPTH { return; }

        let key = board.key();
        let depth = board.move_count();
        let entry = Entry::pack(kind, key, score, depth);

        let hash = self.hash(key);
        if entry.is_superior_to(self.table[hash]) {
            self.table[hash] = entry;
        }
    }

    pub fn get<B: HashBoard>(&self, board: &B) -> Option<(EntryKind, isize)> {
        let key = board.key();
        let hash = self.hash(key);
        let entry = self.table[hash];
        let (kind, other_key, score, _) = entry.unpack()?;
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

    pub fn clear(&mut self) {
        self.table.fill(Entry::EMPTY);
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
            let depth = board.move_count();
            let entry = Entry::pack(kind, key, score, depth);
            let (kind_2, key_2, score_2, depth_2) = entry.unpack().unwrap();
            assert_eq!(key, key_2, "(unpack∘pack)(key) != key for {key}");
            assert_eq!(score, score_2, "(unpack∘pack)(score) != score for {score}");
            assert_eq!(depth, depth_2, "(unpack∘pack)(depth) != depth for {depth}");
        }
    }
}
