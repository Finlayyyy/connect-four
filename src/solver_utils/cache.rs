use crate::board::HashBoard;
use crate::solver_utils::{Position, position};

use ahash::AHasher;
use std::marker::PhantomData;
use std::hash::Hasher;
use std::num::NonZeroU64;
use std::cmp::{min, max};

/// Very large prime number for hash table size
/// * sizeof::<u64>() = ~400MB
const LARGE_SIZE: usize = 50_331_653;

// Relatively small prime number for smaller hash table
// * sizeof::<u64>() = ~2MB
const SMALL_SIZE: usize = 393241;

/// Any entry with a greater depth will not be inserted
const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

const DEPTH_DIFF: u32 = 5;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Entry(NonZeroU64);

impl Entry {
    const KEY_BITS: u64 = 49;
    const EVAL_BITS: u64 = 6;

    const KEY_MASK: u64 = (1 << Self::KEY_BITS) - 1;
    const EVAL_MASK: u64 = (1 << Self::EVAL_BITS) - 1;

    const LOWER_OFFSET: u64 = Self::KEY_BITS;
    const UPPER_OFFSET: u64 = Self::EVAL_BITS + Self::LOWER_OFFSET;

    /// Pack the inputs into a single Entry (u64)
    pub fn pack(key: u64, lower: isize, upper: isize) -> Self {
        debug_assert!(key & (!Self::KEY_MASK) == 0);
        let lower = u64::try_from(lower - position::MIN_EVAL).unwrap();
        debug_assert!(lower & (!Self::EVAL_MASK) == 0);
        let upper = u64::try_from(upper - position::MIN_EVAL).unwrap();
        debug_assert!(upper & (!Self::EVAL_MASK) == 0);

        let entry = key // must be non-zero
            | (lower << Self::LOWER_OFFSET)
            | (upper << Self::UPPER_OFFSET);
        Entry(unsafe { NonZeroU64::new_unchecked(entry) })
    }

    /// Unpack the Entry and return the orignal elements
    pub fn unpack(&self) -> (u64, isize, isize) {
        let entry = u64::from(self.0);
        let key = entry & Self::KEY_MASK;

        let lower = (entry >> Self::LOWER_OFFSET) & Self::EVAL_MASK;
        let lower = isize::try_from(lower).unwrap() + position::MIN_EVAL;
        let upper = (entry >> Self::UPPER_OFFSET) & Self::EVAL_MASK;
        let upper = isize::try_from(upper).unwrap() + position::MIN_EVAL;

        (key, lower, upper)
    }

    pub fn improve_with(self, new: Self) -> Self{
        let (old_key, old_lower, old_upper) = self.unpack();
        let (new_key, new_lower, new_upper) = new.unpack();

        if old_key != new_key {
            if new_key.count_ones() <= DEPTH_DIFF + old_key.count_ones() {
                return new;
            } else {
                return self;
            }
        }
        let lower = max(old_lower, new_lower);
        let upper = min(old_upper, new_upper);
        Entry::pack(new_key, lower, upper)
    }
}

// Hash the given `key` into an index into
/// the inner table with `size` buckets
fn hash(key: u64, size: usize) -> usize {
    let mut hasher = AHasher::default();
    hasher.write_u64(key);
    (hasher.finish() as usize) % size
}

/// Transposition Table Cache
/// Can store a HashBoard with a key of at most 49 bits
pub struct Cache<B> {
    table: Vec<Option<Entry>>,
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

    /// Clear the table by filling it with empty entries
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
        let entry = self.table[hash]?;
        let (other_key, lower, upper) = entry.unpack();
        match key == other_key {
            true => Some((lower, upper)),
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

    /// Insert the board into the cache with lower and upper bound.
    /// MIN_EVAL <= eval <= MAX_EVAL
    pub fn insert(&mut self, board: &B, lower: isize, upper: isize) {
        if board.move_count() > MAX_CACHE_DEPTH { return; }

        let key = board.key();
        let entry = Entry::pack(key, lower, upper);

        let hash = hash(key, self.size);
        if let Some(old) = self.table[hash] {
            self.table[hash] = Some(old.improve_with(entry));
        } else {
            self.table[hash] = Some(entry);
        }

    }

    /// Given a board with an initial alpha, best eval (updated alpha)
    /// and beta bound, inserts an entry into the hash table with the
    /// appropriate bounds.
    #[inline(always)]
    pub fn insert_bounded(
        &mut self,
        board: &B,
        prev_alpha: isize,
        best: isize,
        beta: isize,
    ) {
        let mut lower = position::MIN_EVAL;
        let mut upper = position::MAX_EVAL;
        if best <= prev_alpha {
            upper = best;
        } else if best >= beta {
            lower = best;
        } else {
            lower = best;
            upper = best;
        };
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

    #[test]
    fn pack_unpack() {
        for (moves, eval) in &*END_EASY {
            let board = BitCols::from_moves(moves);
            let key = board.key();
            let lower = eval - 1;
            let upper = eval + 1;
            let entry = Entry::pack(key, lower, upper);
            let (key_2, lower_2, upper_2) = entry.unpack();
            assert_eq!(key, key_2, "(unpack∘pack)(key) != key for {key}");
            assert_eq!(lower, lower_2, "(unpack∘pack)(lower) != lower for {lower}");
            assert_eq!(upper, upper_2, "(unpack∘pack)(upper) != upper for {upper}");
        }
    }

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
