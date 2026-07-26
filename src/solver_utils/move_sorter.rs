use std::{mem::MaybeUninit, slice};

pub struct MoveSorter<const N: usize, K: Ord, T> {
    len: usize,
    elems: [MaybeUninit<(K, T)>; N],
}

impl<const N: usize, K: Ord, T> MoveSorter<N, K, T> {
    const UNINIT: MaybeUninit<(K, T)> = MaybeUninit::uninit();
    pub fn new() -> Self {
        MoveSorter {
            len: 0,
            elems: [Self::UNINIT; N],
        }
    }

    pub fn singleton(key: K, elem: T) -> Self {
        let mut elems = [Self::UNINIT; N];
        elems[0] = MaybeUninit::new((key, elem));
        MoveSorter { len: 1, elems }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push_sorting(&mut self, score: K, elem: T) {
        debug_assert!(self.len < N);
        let mut i = self.len;
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

impl<const N: usize, K: Ord, T> FromIterator<(K, T)> for MoveSorter<N, K, T> {
    fn from_iter<I: IntoIterator<Item = (K, T)>>(iter: I) -> Self {
        let mut sorter = Self::new();
        let mut iter = iter.into_iter();
        while let Some((key, elem)) = iter.next() {
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
        self.elems
            .into_iter()
            .take(self.len)
            .map(|x| unsafe { x.assume_init().1 })
    }
}
