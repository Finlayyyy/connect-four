use crate::board::HashBoard;
use crate::solver_utils::{Position, position};

use ahash::AHasher;
use std::marker::PhantomData;
use std::hash::Hasher;

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

/// Very large prime number for hash table size
/// * sizeof::<u64>() = ~400MB
const LARGE_SIZE: usize = 50_331_653;

// Relatively small prime number for smaller hash table
// * sizeof::<u64>() = ~2MB
const SMALL_SIZE: usize = 393241;

/// Any entry with a greater depth will not be inserted
const MAX_CACHE_DEPTH: usize = position::MAX_MOVES - 5;

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
    table: Vec<Entry>,
    size: usize,
    pd: PhantomData<B>
}
impl<B> Cache<B> {
    /// Construct a new cache with the given amount
    /// of space
    pub fn new(size: usize) -> Self {
        Cache {
            table: vec![Entry::EMPTY; size],
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
        self.table.fill(Entry::EMPTY);
    }
}

impl<B: HashBoard + Position> Cache<B> {
    /// Tries to get the entry corresponding to the given board,
    /// returning `None` if it doesn't exist, `Some(bound_type, eval)`
    /// otherwise.
    pub fn get(&self, board: &B) -> Option<(BoundType, isize)> {
        let key = board.key();
        let hash = hash(key, self.size);
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
        if board.move_count() > MAX_CACHE_DEPTH { return; }

        let key = board.key();
        let depth = board.move_count();
        let entry = Entry::pack(bound, key, eval, depth);

        let hash = hash(key, self.size);
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
    use crate::bench::*;
    use crate::board::*;

    use std::collections::HashSet;

    use statrs::distribution::ChiSquared;
    use statrs::distribution::ContinuousCDF;
    use statrs::statistics::Distribution;

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
