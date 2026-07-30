use std::mem::{ManuallyDrop, MaybeUninit};

/// Datastructure optimised for a fast push
/// and sort (descending) into_iter.
/// i.e. for algorithms to push possible moves onto
/// and then iterate through them
/// ordered (descending) by some heuristic.
/// Current implementation is a insertionsort on each
/// push which is fast for small arrays and stable
pub struct MoveSorter<const N: usize, K: Ord, T> {
    // SAFETY: All elements in 0..len are guaranteed to be initialised.
    len: usize,
    elems: [MaybeUninit<(K, T)>; N],
}

impl<const N: usize, K: Ord, T> MoveSorter<N, K, T> {
    const UNINIT: MaybeUninit<(K, T)> = MaybeUninit::uninit();

    /// Construct a new empty MoveSorter
    pub fn new() -> Self {
        MoveSorter {
            len: 0,
            elems: [Self::UNINIT; N],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Move sorter containing a single element
    pub fn singleton(key: K, elem: T) -> Self {
        let mut elems = [Self::UNINIT; N];
        elems[0] = MaybeUninit::new((key, elem));
        MoveSorter { len: 1, elems }
    }

    /// Add the elem and sort descending by the given score
    pub fn push_sorting(&mut self, score: K, elem: T) {
        debug_assert!(self.len < N);
        let mut i = self.len;
        // SAFETY: All elements in 0..len are guaranteed to be initialised.
        unsafe {
            while (i > 0) && (self.elems[i - 1].assume_init_ref().0 < score) {
                let prev = self.elems[i - 1].assume_init_read();
                self.elems[i] = MaybeUninit::new(prev);
                i -= 1;
            }
        }
        self.elems[i] = MaybeUninit::new((score, elem));
        self.len += 1;
    }
}

impl<const N: usize, K: Ord, T> Drop for MoveSorter<N, K, T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            // drop all initialised elements
            unsafe { self.elems[i].assume_init_drop(); }
        }
    }
}

impl<const N: usize, K: Ord, T> FromIterator<(K, T)> for MoveSorter<N, K, T> {
    fn from_iter<I: IntoIterator<Item = (K, T)>>(iter: I) -> Self {
        let mut sorter = Self::new();
        for (key, elem) in iter {
            sorter.push_sorting(key, elem);
        }
        sorter
    }
}

impl<const N: usize, K: Ord, T> IntoIterator for MoveSorter<N, K, T> {
    type Item = T;
    type IntoIter = std::iter::Map<
        std::iter::Take<std::array::IntoIter<MaybeUninit<(K, T)>, N>>,
        fn(MaybeUninit<(K, T)>) -> T,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let this = ManuallyDrop::new(self);
        // SAFETY: All elements in 0..len are guaranteed initialised
        // and are moved out of the array. All the remaining elements
        // are Uninit and don't need to be dropped. Because `self`
        // is wrapped in `ManuallyDrop`, its `Drop` impl will not
        // run.
        let elems = unsafe { std::ptr::read(&this.elems) };
        elems
            .into_iter()
            .take(this.len)
            .map(|x| unsafe { x.assume_init_read().1 })
    }
}
