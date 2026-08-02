use crate::board::HashBoard;
use crate::solver_utils::{Position, position};

use ahash::AHasher;
use std::marker::PhantomData;
use std::hash::Hasher;
use std::cmp::{min, max};

/// Very large prime number for hash table size
/// * sizeof::<u64>() = ~400MB
const LARGE_SIZE: usize = 50_331_653;

// Relatively small prime number for smaller hash table
// * sizeof::<u64>() = ~2MB
const SMALL_SIZE: usize = 393241;

/// Any entry with a greater depth will not be inserted
const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

/// An older entry must have a depth (move count)
/// `old <= new - DEPTH_DIFF` to avoid being replaced.
const DEPTH_DIFF: u16 = 10;

// Hash the given `key` into an index into
/// the inner table with `size` buckets
fn hash(key: u64, size: usize) -> usize {
    (key as usize) % size
    // let mut hasher = AHasher::default();
    // hasher.write_u64(key);
    // (hasher.finish() as usize) % size
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry {
    /// Board key (hash)
    pub key: u32,
    /// Entry depth (move count)
    pub depth: u16,
    /// Lower bound evaluation
    pub lower: i8,
    /// Upper bound evaluation
    pub upper: i8,
}

impl Entry {
    pub fn new(key: u64, depth: usize, lower: isize, upper: isize) -> Self {
        Entry {
            key: key as u32,
            depth: u16::try_from(depth).unwrap(),
            lower: i8::try_from(lower).unwrap(),
            upper: i8::try_from(upper).unwrap()
        }
    }
    /// An empty entry
    pub const fn empty() -> Self {
        Entry { key: 0, depth: 0, lower: i8::MIN, upper: i8::MAX }
    }
    /// Is the entry empty
    pub const fn is_empty(&self) -> bool {
        self.lower == i8::MIN && self.upper == i8::MAX
    }
    /// Returns `None` if the entry is empty, otherwise
    /// `Some(self)`.
    pub const fn nonempty(self) -> Option<Self> {
        if self.is_empty() {
            None
        } else {
            Some(self)
        }
    }

    /// Given a hash collision with pre-existing entry and a new entry,
    /// choose which entry should take the place.
    pub fn consider_replace(&mut self, new: Self) {
        debug_assert!(!new.is_empty());
        if self.is_empty() {
            *self = new;
            return;
        }

        // Determine which key (board) is more valuable to
        // have in the cache.
        if self.key != new.key {
            if new.depth <= DEPTH_DIFF + self.depth {
                *self = new;
            }
        } else {
            debug_assert!(self.lower <= new.lower || self.upper >= new.upper);
            self.lower = max(self.lower, new.lower);
            self.upper = min(self.upper, new.upper);
        }
    }
}

/// Transposition Table Cache
/// Can store a HashBoard with a key of at most 49 bits
pub struct Cache<B> {
    table: Vec<Entry>,
    size: usize,
    pd: PhantomData<B>
}

impl<B> Cache<B> {
    /// Construct a new cache with the given amount
    /// of space
    pub fn new(size: usize) -> Self {
        Cache {
            table: vec![Entry::empty(); size],
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
        self.table.fill(Entry::empty());
    }
}

impl<B: HashBoard + Position> Cache<B> {
    /// Tries to get the entry corresponding to the given board,
    /// returning `None` if it doesn't exist, `Some(lower, upper)`
    /// otherwise.
    pub fn get(&self, board: &B) -> Option<(isize, isize)> {
        let key = board.key();
        let hash = hash(key, self.size);

        let entry = self.table[hash].nonempty()?;
        match key as u32 == entry.key {
            true => Some((entry.lower as isize, entry.upper as isize)),
            false => None,
        }
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
        if board.move_count() > MAX_CACHE_DEPTH { return; }

        let key = board.key();
        let entry = Entry::new(key, board.move_count(), lower, upper);

        let hash = hash(key, self.size);
        self.table[hash].consider_replace(entry);
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
        let mut lower = i8::MIN as isize;
        let mut upper = i8::MAX as isize;
        if best <= prev_alpha {
            upper = best;
        } else if best >= beta {
            lower = best;
        } else {
            lower = best;
            upper = best;
        }
        self.insert(board, lower, upper);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::bench::*;
    use crate::board::*;

    use std::collections::HashSet;

    use statrs::distribution::ChiSquared;
    use statrs::distribution::ContinuousCDF;
    use statrs::statistics::Distribution;

    fn chi_squared_test<B: HashBoard + CloneBoard + Position>(size: usize) {
        let testsets = [&*END_EASY, &*MIDDLE_EASY, &*MIDDLE_MEDIUM,
            &*BEGIN_EASY, &*BEGIN_MEDIUM, &*BEGIN_HARD, &*SOLVE
        ];
        let moveset = testsets.iter()
            .map(|testset|
                testset.iter().map(|(moves, _)| moves))
            .flatten();

        // ( (#positions(movesets)≈6000)
        // * pow(column::COUNT, DEPTH=4) ) ≈ 10e6
        const DEPTH: usize = 4;

        let mut seen = HashSet::new();
        let mut table: Vec<u32> = vec![0; size];
        let mut visit = |pos: &B| {
            let key = pos.key();
            let idx = hash(key, size);

            if seen.get(&key).is_none() {
                table[idx] += 1;
                seen.insert(key);
            }
        };

        for moves in moveset {
            let board = B::from_moves(moves);
            board.dfs(board.curr(), DEPTH, &mut visit);
        }

        let n = table.iter().sum::<u32>();
        let e = (n as f64) / (size as f64);
        let df = size - 1;

        let chi_sq = e.recip() * table.iter()
            .map(|&o| o * o)
            .sum::<u32>() as f64 - n as f64;

        let dist = ChiSquared::new(df as f64).unwrap();
        let p = dist.sf(chi_sq);

        if p < 0.05 {
            let min = table.iter().min().unwrap();
            let max = table.iter().max().unwrap();

            let mu = dist.mean().unwrap();
            let sigma = dist.std_dev().unwrap();

            println!("Failed chi-squared test.");
            println!("n = {n}, size = {size}");
            println!("e = {e}");
            println!("min = {min}, max = {max}");
            println!("chi_sq = {chi_sq}");
            println!("chi_mean = {}, chi_stddev = {}", mu, sigma);
            println!("z-score = {}", (chi_sq - mu)/sigma);

            println!("p = {p}");
            panic!("p < 0.05, failed for size: {size}");
        }
    }

    #[test]
    fn chi_squared_bitboard() {
        chi_squared_test::<BitBoard>(SMALL_SIZE);
        chi_squared_test::<BitBoard>(LARGE_SIZE);
    }

    #[test]
    fn chi_squared_bitcols() {
        chi_squared_test::<BitCols>(SMALL_SIZE);
        chi_squared_test::<BitCols>(LARGE_SIZE);
    }
}
