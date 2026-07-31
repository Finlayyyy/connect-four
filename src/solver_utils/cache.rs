use crate::board::HashBoard;
use crate::solver_utils::{Position, position};

use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundType {
    Lower,
    Upper,
    Exact,
}

impl std::ops::Neg for BoundType {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            BoundType::Lower => BoundType::Upper,
            BoundType::Exact => BoundType::Exact,
            BoundType::Upper => BoundType::Lower
        }
    }
}

impl BoundType {
    /// Bitrep of an EMPTY BoundType
    pub const EMPTY: u64 = 0;

    /// Pack into 2 bits
    pub const fn pack(self) -> u64 {
        match self {
            BoundType::Lower => 0b01,
            BoundType::Upper => 0b10,
            BoundType::Exact => 0b11,
        }
    }
    /// Try to unpack the 2 lowest bits into an BoundType
    pub const fn unpack(entry: u64) -> Option<Self> {
        match entry {
            0b00 => None,
            0b01 => Some(BoundType::Lower),
            0b10 => Some(BoundType::Upper),
            0b11 => Some(BoundType::Exact),
            _ => panic!("Invalid packed BoundType"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry(u64);

impl Entry {
    const BOUND_BITS: u64 = 2;
    const KEY_BITS: u64 = 49;
    const EVAL_BITS: u64 = 6;
    const DEPTH_BITS: u64 = 6;

    const BOUND_MASK: u64 = (1 << Self::BOUND_BITS) - 1;
    const KEY_MASK: u64 = (1 << Self::KEY_BITS) - 1;
    const EVAL_MASK: u64 = (1 << Self::EVAL_BITS) - 1;
    const DEPTH_MASK: u64 = (1 << Self::DEPTH_BITS) - 1;

    const KEY_OFFSET: u64 = Self::BOUND_BITS;
    const EVAL_OFFSET: u64 = Self::KEY_BITS + Self::KEY_OFFSET;
    const DEPTH_OFFSET: u64 = Self::EVAL_BITS + Self::EVAL_OFFSET;

    /// An empty entry
    pub const EMPTY: Self = Entry(BoundType::EMPTY);

    /// Pack the inputs into a single Entry (u64)
    pub fn pack(bound: BoundType, key: u64, eval: isize, depth: usize) -> Self {
        let bound = bound.pack();
        debug_assert!(bound & (!Self::BOUND_MASK) == 0);
        debug_assert!(key & (!Self::KEY_MASK) == 0);
        let eval = u64::try_from(eval - position::MIN_EVAL).unwrap();
        debug_assert!(eval & (!Self::EVAL_MASK) == 0);
        let depth = u64::try_from(depth).unwrap();
        debug_assert!(depth & (!Self::DEPTH_MASK) == 0);

        Entry(bound | (key << Self::KEY_OFFSET) | (eval << Self::EVAL_OFFSET) | (depth << Self::DEPTH_OFFSET))
    }

    /// Unpack the Entry and return the orignal elements
    pub fn unpack(&self) -> Option<(BoundType, u64, isize, usize)> {
        let entry = self.0;
        let bound = BoundType::unpack(self.0 & Self::BOUND_MASK)?;
        let key = (entry >> Self::KEY_OFFSET) & Self::KEY_MASK;
        let eval = (entry >> Self::EVAL_OFFSET) & Self::EVAL_MASK;
        let eval = isize::try_from(eval).unwrap() + position::MIN_EVAL;
        let depth = (entry >> Self::DEPTH_OFFSET) & Self::DEPTH_MASK;
        let depth = usize::try_from(depth).unwrap();
        Some((bound, key, eval, depth))
    }
}

/// Transposition Table Cache
/// Can store a HashBoard with a key of at most 49 bits
pub struct Cache<B> {
    table: Vec<Entry>,
    max_count: u64,
    pd: PhantomData<B>
}
impl<B> Cache<B> {
    /// Very large prime number for hash table size
    /// * sizeof::<u64>() = ~400MB
    const LARGE_SIZE: usize = 50331653;

    // Relatively small prime number for smaller hash table
    // * sizeof::<u64>() = ~2MB
    const SMALL_SIZE: usize = 393241;

    /// Any entry with a greater depth will not be inserted
    const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

    /// Construct a new cache with the given amount
    /// of space
    pub fn new(size: usize) -> Self {
        Cache {
            table: vec![Entry::EMPTY; size],
            max_count: size as u64,
            pd: PhantomData
        }
    }

    /// Construct a new cache with a large amount
    /// of space.
    pub fn new_large() -> Self { Self::new(Self::LARGE_SIZE) }

    /// Construct a new cache with a small amount
    /// of space.
    pub fn new_small() -> Self { Self::new(Self::SMALL_SIZE) }

    /// Clear the table by filling it with EMPTY entries
    pub fn clear(&mut self) {
        self.table.fill(Entry::EMPTY);
    }

    /// The size of the table in bytes
    pub fn size_of(&self) -> usize {
        size_of::<Entry>() * self.table.len()
    }
}


impl<B: HashBoard + Position> Cache<B> {
    /// Hash the given key into an index into
    /// the inner table
    fn hash(&self, key: u64) -> usize {
        (key % self.max_count) as usize
    }

    /// Tries to get the entry corresponding to the given board,
    /// returning `None` if it doesn't exist, `Some(bound_type, eval)`
    /// otherwise.
    pub fn get(&self, board: &B) -> Option<(BoundType, isize)> {
        let key = board.key();
        let hash = self.hash(key);
        let entry = self.table[hash];
        let (bound, other_key, eval, _) = entry.unpack()?;
        match key == other_key {
            true => Some((bound, eval)),
            false => None,
        }
    }

    /// Search the cache for an entry for board and, if found,
    /// update the given bounds and return them as (alpha, beta)
    #[inline(always)]
    pub fn get_and_bound(&self, board: &B, alpha: isize, beta: isize) -> (isize, isize) {
        match self.get(board) {
            Some((BoundType::Lower, eval)) if eval > alpha => (eval, beta),
            Some((BoundType::Upper, eval)) if eval < beta => (alpha, eval),
            Some((BoundType::Exact, eval)) => (eval, eval),
            _ => (alpha, beta)
        }
    }

    /// Given a hash collision with pre-existing entry and a new entry,
    /// choose which entry should take the place.
    fn choose_entry(old: Entry, new: Entry) -> Entry {
        let Some((old_bound, old_key, old_eval, old_depth)) = old.unpack() else {
            return new;
        };
        let Some((new_bound, new_key, new_eval, new_depth)) = new.unpack() else {
            return old;
        };

        if old_key != new_key {
            if new_depth <= 10  + old_depth {
                return new;
            } else {
                return old;
            }
        }
        if old_bound == BoundType::Exact { return old; }
        if new_bound == BoundType::Exact { return new; }

        if old_bound == -new_bound && old_eval == new_eval {
            return Entry::pack(BoundType::Exact, new_key, new_eval, new_depth);
        }

        new
    }

    /// Insert the board into the cache with bound type and eval.
    /// MIN_EVAL <= eval <= MAX_EVAL
    pub fn insert(&mut self, bound: BoundType, board: &B, eval: isize) {
        if board.move_count() > Self::MAX_CACHE_DEPTH { return; }

        let key = board.key();
        let depth = board.move_count();
        let entry = Entry::pack(bound, key, eval, depth);

        let hash = self.hash(key);
        let entry = Self::choose_entry(self.table[hash], entry);
        self.table[hash] = entry;
    }

    /// Given a board with an initial alpha, best eval (updated alpha)
    /// and beta bound, inserts an entry into the hash table with the
    /// appropriate bound type.
    #[inline(always)]
    pub fn insert_bounded(
        &mut self,
        board: &B,
        prev_alpha: isize,
        best: isize,
        beta: isize,
    ) {
        let bound = if best <= prev_alpha {
            BoundType::Upper
        } else if best >= beta {
            BoundType::Lower
        } else {
            BoundType::Exact
        };
        self.insert(bound, board, best);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::END_EASY;
    use crate::board::*;

    #[test]
    fn pack_unpack() {
        for (moves, eval) in &*END_EASY {
            let bound = BoundType::Exact;
            let board = BitCols::from_moves(moves);
            let key = board.key();
            let depth = board.move_count();
            let entry = Entry::pack(bound, key, *eval, depth);
            let (_kind_2, key_2, eval_2, depth_2) = entry.unpack().unwrap();
            assert_eq!(key, key_2, "(unpack∘pack)(key) != key for {key}");
            assert_eq!(*eval, eval_2, "(unpack∘pack)(eval) != eval for {eval}");
            assert_eq!(depth, depth_2, "(unpack∘pack)(depth) != depth for {depth}");
        }
    }
}
