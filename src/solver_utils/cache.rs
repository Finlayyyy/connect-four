use crate::board::HashBoard;
use crate::solver_utils::Position;

use std::marker::PhantomData;
use std::cmp::{min, max};
use std::num::NonZeroU32;

/// Very large prime number for hash table size
/// * sizeof::<u64>() = ~400MB
const LARGE_SIZE: usize = 50331653;

// Relatively small prime number for smaller hash table
// * sizeof::<u64>() = ~2MB
const SMALL_SIZE: usize = 393241;

// Hash the given `key` into an index into
/// the inner table with `size` buckets
fn hash(key: u64, size: usize) -> usize {
    (key as usize) % size
}

/// A cache entry for a board and evaluation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    /// Board key (hash)
    key: NonZeroU32,
    /// Entry depth (move count)
    depth: u16,
    /// Lower bound evaluation
    lower: i8,
    /// Upper bound evaluation
    upper: i8,
}

impl Entry {
    pub fn new(key: u64, depth: usize, lower: isize, upper: isize) -> Self {
        Entry {
            key: unsafe { NonZeroU32::new_unchecked(key as u32) },
            depth: u16::try_from(depth).unwrap(),
            lower: i8::try_from(lower).unwrap(),
            upper: i8::try_from(upper).unwrap()
        }
    }
}

/// Two-tiered entry, containing a deep entry (lowest move count)
/// and a recent entry (most recent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DoubleEntry {
    /// Entry (lowest move count)
    deep: Entry,
    /// Entry (most recent)
    recent: Entry
}

impl DoubleEntry {
    pub fn new(entry: Entry) -> Self {
        DoubleEntry {
            deep: entry,
            recent: entry
        }
    }

    pub fn get(&self, key: u64) -> Option<(isize, isize)> {
        if u32::from(self.deep.key) == key as u32 {
            return Some((self.deep.lower as isize, self.deep.upper as isize))
        }

        if u32::from(self.recent.key) == key as u32 {
            return Some((self.recent.lower as isize, self.recent.upper as isize))
        }

        None
    }

    /// Replace by the lowest move count
    fn improve_deep(old: &mut Entry, new: Entry) {
        if old.key == new.key {
            old.lower = max(old.lower, new.lower);
            old.upper = min(old.upper, new.upper);
        } else if new.depth <= old.depth {
            *old = new;
        }
    }

    /// Always replace the old entry
    fn improve_recent(old: &mut Entry, new: Entry) {
        if old.key == new.key {
            old.lower = max(old.lower, new.lower);
            old.upper = min(old.upper, new.upper);
        } else {
            *old = new;
        }
    }

    /// Given a hash collision with pre-existing entry and a new entry,
    /// choose which entry should take the place.
    pub fn improve(&mut self, new: Entry) {
        Self::improve_deep(&mut self.deep, new);
        Self::improve_recent(&mut self.recent, new);
    }
}

/// Transposition Table Cache
/// Can store a HashBoard with a key of at most 49 bits
pub struct Cache<B> {
    table: Vec<Option<DoubleEntry>>,
    size: usize,
    pd: PhantomData<B>
}

impl<B> Cache<B> {
    /// Construct a new cache with the given amount
    /// of space
    pub fn new(size: usize) -> Self {
        Cache {
            table: vec![None; size],
            size,
            pd: PhantomData
        }
    }

    /// Construct a new cache with a large amount
    /// of space.
    pub fn new_large() -> Self { Self::new(LARGE_SIZE) }

    /// Construct a new cache with a small amount
    /// of space.
    pub fn new_small() -> Self { Self::new(SMALL_SIZE) }

    /// Clear the table by filling it with EMPTY entries
    pub fn clear(&mut self) {
        self.table.fill(None);
    }
}

impl<B: HashBoard + Position> Cache<B> {
    /// Tries to get the entry corresponding to the given board,
    /// returning `None` if it doesn't exist, `Some(lower, upper)`
    /// otherwise.
    pub fn get(&self, board: &B) -> Option<(isize, isize)> {
        let key = board.key();
        let hash = hash(key, self.size);

        let entries = self.table[hash]?;
        entries.get(key)
    }

    /// Search the cache for an entry for board and, if found,
    /// update the given bounds and return them as (alpha, beta)
    #[inline(always)]
    pub fn get_and_bound(&self, board: &B, alpha: isize, beta: isize) -> (isize, isize) {
        let Some((lower, upper)) = self.get(board) else {
            return (alpha, beta);
        };
        let alpha = max(alpha, lower);
        let beta = min(beta, upper);
        (alpha, beta)
    }


    /// Insert the board into the cache with lower and upper bound
    /// MIN_EVAL <= eval <= MAX_EVAL
    pub fn insert(&mut self, board: &B, lower: isize, upper: isize) {
        let key = board.key();
        let entry = Entry::new(key, board.move_count(), lower, upper);

        let hash = hash(key, self.size);
        if let Some(old) = &mut self.table[hash] {
            old.improve(entry);
        } else {
            self.table[hash] = Some(DoubleEntry::new(entry));
        }
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
        let mut lower = board.will_lose_eval();
        let mut upper = board.will_win_eval();
        if best <= prev_alpha {
            upper = best; // upper bound
        } else if best >= beta {
            lower = best; // lower bound
        } else {
            // exact bound
            lower = best;
            upper = best;
        }
        self.insert(board, lower, upper);
    }
}
