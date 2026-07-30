use rand::RngExt;
use rand::distr::{Distribution, StandardUniform};

// Rust hackery to allow for assertions on
// const generic parameters
pub enum Assert<const COND: bool> {}
pub trait IsTrue {}
impl IsTrue for Assert<true> {}

/// Unsigned finite natural number type, with values in `[0, N)`.
/// Used for indexing a collection of `N` elements.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct FiniteIndex<const N: usize>(usize);

impl<const N: usize> FiniteIndex<N>
where Assert<{ N > 0 }>: IsTrue
{
    /// `0 ∈ [0, N)` where `N > 0`
    pub const ZERO: FiniteIndex<N> = FiniteIndex(0);
    /// `(N - 1) ∈ [0, N)` where `N > 0`
    pub const MAX: FiniteIndex<N> = FiniteIndex(N - 1);
}

impl<const N: usize> FiniteIndex<N> {
    /// Number of elements in the set `FiniteIndex<N>`
    pub const COUNT: usize = N;

    /// Creates a FiniteIndex from a raw usize value, panicking
    /// when out of bounds.
    #[inline(always)]
    pub const fn raw(value: usize) -> Self {
        debug_assert!(
            value < N,
            "FiniteIndex out of bounds error: value exceeds the maximum allowed"
        );
        FiniteIndex(value)
    }

    #[inline(always)]
    pub const fn to_u64(self) -> u64 {
        self.0 as u64
    }

    /// Shifts the value by the given amount,
    /// staying within bounds by capping/saturating at the edges.
    #[inline(always)]
    pub fn shift(&self, by: isize) -> Self {
        FiniteIndex(self.0.saturating_add_signed(by).clamp(0, N - 1))
    }

    /// Adds the given amount to the value,
    /// staying within bounds by capping at the maximum.
    #[inline(always)]
    pub fn add_usize(&self, by: usize) -> Self {
        FiniteIndex(self.0.saturating_add(by).min(N - 1))
    }

    /// Subtracts the given amount from the value,
    /// staying within bounds by capping at zero.
    #[inline(always)]
    pub fn sub_usize(&self, by: usize) -> Self {
        FiniteIndex(self.0.saturating_sub(by))
    }

    /// Tries to shift the value by the given amount,
    /// returning None when out of bounds
    #[inline(always)]
    pub fn try_shift(&self, by: isize) -> Option<Self> {
        let val = self.0.checked_add_signed(by)?;
        if val < N {
            Some(FiniteIndex(val))
        } else {
            None
        }
    }

    /// Tries to add the given amount to the value,
    /// returning None when out of bounds
    #[inline(always)]
    pub fn try_add_usize(&self, by: usize) -> Option<Self> {
        if self.0 + by < N {
            Some(FiniteIndex(self.0 + by))
        } else {
            None
        }
    }

    /// Tries to subtract the given amount from the value,
    /// returning None when out of bounds
    #[inline(always)]
    pub fn try_sub_usize(&self, by: usize) -> Option<Self> {
        if self.0 >= by {
            Some(FiniteIndex(self.0 - by))
        } else {
            None
        }
    }
}


/* From and TryFrom */
impl<const N: usize> TryFrom<usize> for FiniteIndex<N> {
    type Error = String;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        if value < N {
            Ok(FiniteIndex(value))
        } else {
            Err(format!(
                "FiniteIndex conversion error. Out of bounds as {} > {}",
                value, N
            ))
        }
    }
}
impl<const N: usize> TryFrom<isize> for FiniteIndex<N> {
    type Error = String;

    fn try_from(value: isize) -> Result<Self, Self::Error> {
        FiniteIndex::try_from(usize::try_from(value).map_err(|e| e.to_string())?)
    }
}
impl<const N: usize> TryFrom<u32> for FiniteIndex<N> {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        FiniteIndex::try_from(usize::try_from(value).map_err(|e| e.to_string())?)
    }
}

/* To and TryTo */
impl<const N: usize> From<FiniteIndex<N>> for usize {
    fn from(value: FiniteIndex<N>) -> Self {
        value.0
    }
}
impl<const N: usize> From<FiniteIndex<N>> for u8
where
    Assert<{ N <= 1 << 8 }>: IsTrue,
{
    fn from(value: FiniteIndex<N>) -> Self {
        u8::try_from(value.0).unwrap()
    }
}
impl<const N: usize> From<FiniteIndex<N>> for u32
where
    Assert<{ N <= 1 << 32 }>: IsTrue,
{
    fn from(value: FiniteIndex<N>) -> Self {
        u32::try_from(value.0).unwrap()
    }
}
impl<const N: usize> From<FiniteIndex<N>> for u64 {
    fn from(value: FiniteIndex<N>) -> Self {
        u64::try_from(value.0).unwrap()
    }
}
// isize must at least be 16 bits
impl<const N: usize> From<FiniteIndex<N>> for isize
where Assert<{ N <= 1 << 8 }>: IsTrue,
{
    fn from(value: FiniteIndex<N>) -> Self {
        isize::from(u8::from(value))
    }
}

impl<const N: usize> Distribution<FiniteIndex<N>> for StandardUniform
where Assert<{ N > 0 }>: IsTrue
{
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> FiniteIndex<N> {
        rng.random_range(0..N).try_into().unwrap()
    }
}
