//! Pluggable pair storage for the borrowing query string builder.

use crate::Part;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{self, Display, Formatter};

/// A borrowed key-value pair as stored by [`QueryString`](crate::QueryString).
pub type Pair<'a> = (Part<'a>, Part<'a>);

/// Storage backing the pairs of a [`QueryString`](crate::QueryString).
///
/// Implemented for [`Vec`] (the default, growable storage) and for
/// [`InlinePairs`] (fixed-capacity storage living inline, e.g. on the stack).
/// Implement it for other collections — such as `heapless::Vec` or
/// `arrayvec::ArrayVec` — to plug them into the builder.
pub trait PairStorage<'a> {
    /// Returns the stored pairs in insertion order.
    fn as_slice(&self) -> &[Pair<'a>];

    /// Appends a pair, handing it back if the storage is full.
    fn try_push(&mut self, pair: Pair<'a>) -> Result<(), Pair<'a>>;
}

#[cfg(feature = "alloc")]
impl<'a> PairStorage<'a> for Vec<Pair<'a>> {
    #[inline]
    fn as_slice(&self) -> &[Pair<'a>] {
        self
    }

    #[inline]
    fn try_push(&mut self, pair: Pair<'a>) -> Result<(), Pair<'a>> {
        self.push(pair);
        Ok(())
    }
}

/// Fixed-capacity pair storage holding up to `N` pairs inline.
///
/// Use it via [`QueryString::inline`](crate::QueryString::inline) to build a
/// query string without any heap allocation at all.
///
/// ## Example
///
/// ```
/// use query_string_builder::QueryString;
///
/// let qs = QueryString::inline::<2>()
///     .with("q", "apple")
///     .with("category", "fruits");
///
/// assert_eq!(qs.to_string(), "?q=apple&category=fruits");
/// ```
#[derive(Clone, Copy)]
pub struct InlinePairs<'a, const N: usize> {
    pairs: [Pair<'a>; N],
    len: usize,
}

impl<const N: usize> InlinePairs<'_, N> {
    /// Placeholder for unused slots; never observable through [`PairStorage`].
    const EMPTY: Pair<'static> = (Part::Str(""), Part::Str(""));

    /// Creates empty inline storage.
    pub const fn new() -> Self {
        Self {
            pairs: [Self::EMPTY; N],
            len: 0,
        }
    }
}

impl<const N: usize> Default for InlinePairs<'_, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, const N: usize> PairStorage<'a> for InlinePairs<'a, N> {
    #[inline]
    fn as_slice(&self) -> &[Pair<'a>] {
        &self.pairs[..self.len]
    }

    #[inline]
    fn try_push(&mut self, pair: Pair<'a>) -> Result<(), Pair<'a>> {
        match self.pairs.get_mut(self.len) {
            Some(slot) => {
                *slot = pair;
                self.len += 1;
                Ok(())
            }
            None => Err(pair),
        }
    }
}

/// The error returned by [`QueryString::try_push`](crate::QueryString::try_push)
/// when fixed-capacity storage is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapacityError;

impl Display for CapacityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("query string pair storage is full")
    }
}

impl Error for CapacityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_push_until_full() {
        let mut storage = InlinePairs::<1>::new();
        assert!(storage.as_slice().is_empty());
        assert!(storage.try_push((Part::Str("a"), Part::Str("1"))).is_ok());
        assert_eq!(storage.as_slice().len(), 1);
        assert!(storage.try_push((Part::Str("b"), Part::Str("2"))).is_err());
        assert_eq!(storage.as_slice().len(), 1);
    }

    #[test]
    fn test_inline_zero_capacity() {
        let mut storage = InlinePairs::<0>::default();
        assert!(storage.try_push((Part::Str("a"), Part::Str("1"))).is_err());
    }

    #[test]
    fn test_capacity_error_display() {
        assert_eq!(
            CapacityError.to_string(),
            "query string pair storage is full"
        );
    }
}
