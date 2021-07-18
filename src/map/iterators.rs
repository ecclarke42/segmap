use core::{
    fmt::{self, Debug},
    iter::{FromIterator, FusedIterator},
};

use super::Key;
use crate::{
    bounds::{EndBound, StartBound},
    Range, RangeMap,
};
// TODO: all doctests

impl<K, V> RangeMap<K, V> {
    /// Gets an iterator over the sorted ranges in the map.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(3, "c");
    /// map.insert(2, "b");
    /// map.insert(1, "a");
    ///
    /// for (key, value) in map.iter() {
    ///     println!("{}: {}", key, value);
    /// }
    ///
    /// let (first_key, first_value) = map.iter().next().unwrap();
    /// assert_eq!((*first_key, *first_value), (1, "a"));
    /// ```
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter(self.map.iter())
    }

    // TODO: iter_in(): Iter but with a subset range

    /// Gets an iterator over the sorted ranges in the map, with mutable values
    ///
    /// Ranges are used as keys and therefore cannot be mutable. To manipulate
    /// the bounds of stored ranges, they must be removed and re-inserted to
    /// ensure bound integrity.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert("a", 1);
    /// map.insert("b", 2);
    /// map.insert("c", 3);
    ///
    /// // add 10 to the value if the key isn't "a"
    /// for (key, value) in map.iter_mut() {
    ///     if key != &"a" {
    ///         *value += 10;
    ///     }
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut(self.map.iter_mut())
    }

    /// Gets an iterator over the range keys of the map (similar to `BTreeMap::keys()`)
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// a.insert(2, "b");
    /// a.insert(1, "a");
    ///
    /// let keys: Vec<_> = a.keys().cloned().collect();
    /// assert_eq!(keys, [1, 2]);
    /// ```
    // pub fn keys(&self) -> Keys<'_, K, V> {
    //     Keys(self.iter())
    // }
    pub fn ranges(&self) -> Ranges<'_, K, V> {
        Ranges(self.iter())
    }

    /// Gets an iterator over the values of the map, in order by their range.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// a.insert(1, "hello");
    /// a.insert(2, "goodbye");
    ///
    /// let values: Vec<&str> = a.values().cloned().collect();
    /// assert_eq!(values, ["hello", "goodbye"]);
    /// ```
    pub fn values(&self) -> Values<'_, K, V> {
        Values(self.iter())
    }

    /// Gets a mutable iterator over the values of the map, in order by their
    /// range.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// a.insert(1, String::from("hello"));
    /// a.insert(2, String::from("goodbye"));
    ///
    /// for value in a.values_mut() {
    ///     value.push_str("!");
    /// }
    ///
    /// let values: Vec<String> = a.values().cloned().collect();
    /// assert_eq!(values, [String::from("hello!"),
    ///                     String::from("goodbye!")]);
    /// ```
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut(self.iter_mut())
    }

    /// Returns the number of ranges in the map.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the map contains no ranges.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// let mut a = BTreeMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    // fn range_bounds(&self) -> R?

    // TODO: and type
    // pub fn iter_complement(&self) -> impl Iterator<Item = Range<K>> {
    //     todo!()
    // }

    /// Gets an iterator over all maximally-sized gaps between ranges in the map
    ///
    /// NOTE: Empty regions before and after those stored in this map (i.e.
    /// before the first range and after the last range) will not be included
    /// in this iterator
    pub fn gaps(&self) -> Gaps<'_, K, V> {
        Gaps {
            iter: self.iter(),
            prev: None,
        }
    }

    // TODO: may simplify set_gaps()
    /// Gets an iterator over all maximally-sized gaps between ranges in the map,
    /// further bounded by an outer range
    ///
    /// NOTE: Unlike [`gaps`], the iterator here WILL include regions before and
    /// after those stored in the map, so long as they are included in the outer
    /// range
    pub fn gaps_in<'a, R: 'a + core::ops::RangeBounds<K>>(
        &'a self,
        range: R,
    ) -> GapsIn<'a, K, V, R> {
        // TODO: why can't we borrow start/end and make `bounds` a Range<&'a T>?
        GapsIn {
            iter: self.iter(),
            prev: None,
            bounds: range,
        }
    }
}

impl<K, V> IntoIterator for RangeMap<K, V> {
    type Item = (Range<K>, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.map.into_iter())
    }
}
impl<'a, K, V> IntoIterator for &'a RangeMap<K, V> {
    type Item = (&'a Range<K>, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Iter<'a, K, V> {
        self.iter()
    }
}

impl<R: core::ops::RangeBounds<K>, K: Clone + Ord, V: Clone + Eq> FromIterator<(R, V)>
    for RangeMap<K, V>
{
    fn from_iter<T: IntoIterator<Item = (R, V)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

// TODO: note need for clone (insert) and setting existing
impl<R: core::ops::RangeBounds<K>, K: Clone + Ord, V: Clone + Eq> Extend<(R, V)>
    for RangeMap<K, V>
{
    #[inline]
    fn extend<T: IntoIterator<Item = (R, V)>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |(k, v)| {
            self.set(k, v);
        });
    }

    // #[inline]
    // fn extend_one(&mut self, (k, v): (K, V)) {
    //     self.insert(k, v);
    // }
}
// impl<'a, K: Ord + Copy, V: Copy> Extend<(&'a K, &'a V)> for RangeMap<K, V> {
//     fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: I) {
//         self.extend(iter.into_iter().map(|(&key, &value)| (key, value)));
//     }

//     // #[inline]
//     // fn extend_one(&mut self, (&k, &v): (&'a K, &'a V)) {
//     //     self.insert(k, v);
//     // }
// }

/// An iterator over the entries of a `RangeMap`.
///
/// This `struct` is created by the [`iter`] method on [`RangeMap`]. See its
/// documentation for more.
///
/// [`iter`]: RangeMap::iter
pub struct Iter<'a, K, V>(alloc::collections::btree_map::Iter<'a, Key<K>, V>);
impl<K: Debug, V: Debug> Debug for Iter<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a Range<K>, &'a V);
    fn next(&mut self) -> Option<(&'a Range<K>, &'a V)> {
        self.0.next().map(|(wrapper, v)| (&wrapper.0, v))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn last(mut self) -> Option<(&'a Range<K>, &'a V)> {
        self.next_back()
    }
    fn min(mut self) -> Option<(&'a Range<K>, &'a V)> {
        self.next()
    }
    fn max(mut self) -> Option<(&'a Range<K>, &'a V)> {
        self.next_back()
    }
}
impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for Iter<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a Range<K>, &'a V)> {
        self.0.next_back().map(|(wrapper, v)| (&wrapper.0, v))
    }
}
impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<'a, K, V> IntoIterator for &'a mut RangeMap<K, V> {
    type Item = (&'a Range<K>, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }
}
/// A mutable iterator over the entries of a `RangeMap`.
///
/// This `struct` is created by the [`iter_mut`] method on [`RangeMap`]. See its
/// documentation for more.
///
/// [`iter_mut`]: RangeMap::iter_mut
pub struct IterMut<'a, K: 'a, V: 'a>(alloc::collections::btree_map::IterMut<'a, Key<K>, V>);
// TODO
impl<K: Debug, V: Debug> Debug for IterMut<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a Range<K>, &'a mut V);

    fn next(&mut self) -> Option<(&'a Range<K>, &'a mut V)> {
        self.0.next().map(|(wrapper, v)| (&wrapper.0, v))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn last(mut self) -> Option<(&'a Range<K>, &'a mut V)> {
        self.next_back()
    }

    fn min(mut self) -> Option<(&'a Range<K>, &'a mut V)> {
        self.next()
    }

    fn max(mut self) -> Option<(&'a Range<K>, &'a mut V)> {
        self.next_back()
    }
}

impl<'a, K: 'a, V: 'a> DoubleEndedIterator for IterMut<'a, K, V> {
    fn next_back(&mut self) -> Option<(&'a Range<K>, &'a mut V)> {
        self.0.next_back().map(|(wrapper, v)| (&wrapper.0, v))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> FusedIterator for IterMut<'_, K, V> {}

// impl<'a, K, V> IterMut<'a, K, V> {
//     /// Returns an iterator of references over the remaining items.
//     #[inline]
//     pub(super) fn iter(&self) -> Iter<'_, K, V> {
//         Iter(self.0.iter())
//     }
// }

/// An owning iterator over the entries of a `RangeMap`.
///
/// This `struct` is created by the [`into_iter`] method on [`RangeMap`]
/// (provided by the `IntoIterator` trait). See its documentation for more.
///
/// [`into_iter`]: IntoIterator::into_iter
pub struct IntoIter<K, V>(alloc::collections::btree_map::IntoIter<Key<K>, V>);
impl<K: Debug, V: Debug> Debug for IntoIter<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (Range<K>, V);
    fn next(&mut self) -> Option<(Range<K>, V)> {
        self.0.next().map(|(wrapper, v)| (wrapper.0, v))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(Range<K>, V)> {
        self.0.next_back().map(|(wrapper, v)| (wrapper.0, v))
    }
}
impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<K, V> FusedIterator for IntoIter<K, V> {}
// impl<K, V> IntoIter<K, V> {
//     #[inline]
//     pub(super) fn iter(&self) -> Iter<'_, K, V> {
//         Iter(self.)
//     }
// }

/// An iterator over the keys of a `RangeMap`.
///
/// This `struct` is created by the [`keys`] method on [`RangeMap`]. See its
/// documentation for more.
///
/// [`keys`]: RangeMap::keys
pub struct Ranges<'a, K: 'a, V: 'a>(Iter<'a, K, V>);
impl<K: Debug, V> Debug for Ranges<'_, K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
impl<'a, K, V> Iterator for Ranges<'a, K, V> {
    type Item = &'a Range<K>;
    fn next(&mut self) -> Option<&'a Range<K>> {
        self.0.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn last(mut self) -> Option<&'a Range<K>> {
        self.next_back()
    }
    fn min(mut self) -> Option<&'a Range<K>> {
        self.next()
    }
    fn max(mut self) -> Option<&'a Range<K>> {
        self.next_back()
    }
}
impl<'a, K, V> DoubleEndedIterator for Ranges<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a Range<K>> {
        self.0.next_back().map(|(k, _)| k)
    }
}
impl<K, V> ExactSizeIterator for Ranges<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<K, V> FusedIterator for Ranges<'_, K, V> {}

impl<K, V> Clone for Ranges<'_, K, V> {
    fn clone(&self) -> Self {
        Ranges(self.0.clone())
    }
}

/// An iterator over the values of a `RangeMap`.
///
/// This `struct` is created by the [`values`] method on [`RangeMap`]. See its
/// documentation for more.
///
/// [`values`]: RangeMap::values
#[derive(Clone)]
pub struct Values<'a, K: 'a, V: 'a>(Iter<'a, K, V>);
// TODO
// impl<K, V: Debug> Debug for Values<'_, K, V> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_list().entries(self.clone()).finish()
//     }
// }
impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<&'a V> {
        self.0.next().map(|(_, v)| v)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn last(mut self) -> Option<&'a V> {
        self.next_back()
    }
}
impl<'a, K, V> DoubleEndedIterator for Values<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.0.next_back().map(|(_, v)| v)
    }
}
impl<K, V> ExactSizeIterator for Values<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<K, V> FusedIterator for Values<'_, K, V> {}

/// A mutable iterator over the values of a `RangeMap`.
///
/// This `struct` is created by the [`values_mut`] method on [`RangeMap`]. See its
/// documentation for more.
///
/// [`values_mut`]: RangeMap::values_mut
pub struct ValuesMut<'a, K: 'a, V: 'a>(IterMut<'a, K, V>);
// TODO
// impl<K, V: Debug> Debug for ValuesMut<'_, K, V> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_list()
//             .entries(self.iter().map(|(_, val)| val))
//             .finish()
//     }
// }
impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
    type Item = &'a mut V;
    fn next(&mut self) -> Option<&'a mut V> {
        self.0.next().map(|(_, v)| v)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn last(mut self) -> Option<&'a mut V> {
        self.next_back()
    }
}
impl<'a, K, V> DoubleEndedIterator for ValuesMut<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a mut V> {
        self.0.next_back().map(|(_, v)| v)
    }
}
impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}

pub struct Gaps<'a, K, V> {
    iter: Iter<'a, K, V>,
    prev: Option<&'a Range<K>>,
}

// TODO: document panics in Gaps

impl<'a, K, V> Iterator for Gaps<'a, K, V>
where
    K: Ord + Clone,
{
    type Item = Range<K>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((next, _)) = self.iter.next() {
            if let Some(prev) = self.prev {
                // Get the adjacent bound to the end of the previous range

                let start = prev.bound_after()?.cloned(); // If none, no more gaps (this extends forwards to infinity)
                let end = next
                    .bound_before()
                    .expect("Unbounded internal range in RangeMap")
                    .cloned();
                self.prev = Some(next);
                Some(Range { start, end })
            } else {
                // No previous bound means first gap

                // Get the adjacent bound to the end of the first range
                let start = next.bound_after()?.cloned(); // If none, no more gaps (this extends forwards to infinity)

                // Check if we have another range
                if let Some((next, _)) = self.iter.next() {
                    // Store the end of the next segment for next iteration
                    let end = next
                        .bound_before()
                        .expect("Unbounded internal range in RangeMap")
                        .cloned();

                    self.prev = Some(next);
                    Some(Range { start, end })
                } else {
                    // Only one item (no gaps)
                    None
                }
            }
        } else {
            None
        }
    }
}

impl<K: Clone + Ord, V> FusedIterator for Gaps<'_, K, V> {}

pub struct GapsIn<'a, K, V, R> {
    iter: Iter<'a, K, V>,
    prev: Option<&'a Range<K>>,
    bounds: R,
}

// TODO: document panics in Gaps

impl<'a, K, V, R> Iterator for GapsIn<'a, K, V, R>
where
    K: Ord,
    R: core::ops::RangeBounds<K>,
{
    type Item = Range<&'a K>;
    fn next(&mut self) -> Option<Self::Item> {
        todo!();
        // TODO

        // if let Some((next, _)) = self.iter.next() {
        //     if let Some(prev) = self.prev {
        //         // Get the adjacent bound to the end of the previous range

        //         let start = prev.bound_after()?.cloned(); // If none, no more gaps (this extends forwards to infinity)
        //         let end = next
        //             .bound_before()
        //             .expect("Unbounded internal range in RangeMap")
        //             .cloned();
        //         self.prev = Some(next);
        //         Some(Range { start, end })
        //     } else {
        //         // No previous bound means first gap

        //         // Get the adjacent bound to the end of the first range
        //         let start = next.bound_after()?.cloned(); // If none, no more gaps (this extends forwards to infinity)

        //         // Check if we have another range
        //         if let Some((next, _)) = self.iter.next() {
        //             // Store the end of the next segment for next iteration
        //             let end = next
        //                 .bound_before()
        //                 .expect("Unbounded internal range in RangeMap")
        //                 .cloned();

        //             self.prev = Some(next);
        //             Some(Range { start, end })
        //         } else {
        //             // Only one item (no gaps)
        //             None
        //         }
        //     }
        // } else {
        //     None
        // }
    }
}

impl<K: Clone + Ord, V, R: core::ops::RangeBounds<K>> FusedIterator for GapsIn<'_, K, V, R> {}
