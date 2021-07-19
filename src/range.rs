use crate::bounds::{EndBound, StartBound};
use core::{
    cmp::Ordering,
    fmt::Debug,
    ops::Bound::{self, *},
};

/// Monotonically increasing segment, for use as a concrete range type in
/// [`RangeMap`].
#[derive(Clone, Copy, Hash, Eq)]
pub struct Range<T> {
    pub(crate) start: StartBound<T>,
    pub(crate) end: EndBound<T>,
}

impl<T> core::ops::RangeBounds<T> for Range<T> {
    fn start_bound(&self) -> Bound<&T> {
        self.start.as_bound_inner_ref()
    }
    fn end_bound(&self) -> Bound<&T> {
        self.end.as_bound_inner_ref()
    }
}

impl<T> core::ops::RangeBounds<T> for &Range<T> {
    fn start_bound(&self) -> Bound<&T> {
        self.start.as_bound_inner_ref()
    }
    fn end_bound(&self) -> Bound<&T> {
        self.end.as_bound_inner_ref()
    }
}

impl<T: Debug> Debug for Range<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.start.0 {
            Unbounded => write!(f, "(-∞, ")?,
            Excluded(t) => write!(f, "({:?}, ", t)?,
            Included(t) => write!(f, "[{:?}, ", t)?,
        }
        match &self.end.0 {
            Unbounded => write!(f, "∞)"),
            Excluded(t) => write!(f, "{:?})", t),
            Included(t) => write!(f, "{:?}]", t),
        }
    }
}

impl<T> Range<T> {
    /// Construct a new segment from range bounds
    ///
    /// If the range given is backwards (decreasing), it will be reversed
    ///
    /// ## Weird Ranges
    ///
    /// Oddly constructed ranges will be coerced to "sane" structure:
    /// - Point ranges (where both start and end are bounded to the same value
    /// and at least one is included) will be coerced to (Included, Included)
    ///
    /// ## Panics
    ///
    /// Only one range is too strange to be included here. If you pass a point
    /// range with both the start and end excluded, this will panic, as the
    /// range is impossible to evaluate
    ///
    /// ```
    /// use rangemap::{Range, Bound};
    /// use core::ops::RangeBounds;
    ///
    /// let r = Range::new(0..5);
    /// assert_eq!(r.start_bound(), Bound::Included(&0));
    /// assert_eq!(r.end_bound(), Bound::Excluded(&5));
    /// ```
    pub fn new<R: core::ops::RangeBounds<T>>(r: R) -> Self
    where
        T: Ord + Clone,
    {
        let start = bound_cloned(r.start_bound());
        let end = bound_cloned(r.end_bound());
        match (&start, &end) {
            (Included(s), Included(e))
            | (Included(s), Excluded(e))
            | (Excluded(s), Included(e))
            | (Excluded(s), Excluded(e)) => match s.cmp(e) {
                // If equal, coerce to included
                Ordering::Equal => Self {
                    start: StartBound(Included(s.clone())),
                    end: EndBound(Included(e.clone())),
                },
                Ordering::Less => Self {
                    start: StartBound(start),
                    end: EndBound(end),
                },
                // If backwards, flip bounds
                Ordering::Greater => Self {
                    start: StartBound(end),
                    end: EndBound(start),
                },
            },

            // Otherwise, only one side has a value
            _ => Self {
                start: StartBound(start),
                end: EndBound(end),
            },
        }
    }

    /// Construct a new [`Range`] that spans all possible values
    ///
    /// This is the same as `Range::new(..)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use rangemap::{Range, Bound, RangeBounds};
    ///
    /// let r = Range::<u32>::full();
    /// assert_eq!(r.start_bound(), Bound::Unbounded);
    /// assert_eq!(r.end_bound(), Bound::Unbounded);
    /// ```
    pub fn full() -> Self {
        Self {
            start: StartBound(Unbounded),
            end: EndBound(Unbounded),
        }
    }

    /// Constructs a new point segement at the given value
    ///
    /// # Examples
    ///
    /// ```
    /// use rangemap::{Range, RangeBounds, Bound};
    ///
    /// let r = Range::point(5);
    /// assert_eq!(r.start_bound(), Bound::Included(&5));
    /// assert_eq!(r.end_bound(), Bound::Included(&5));
    /// ```
    pub fn point(value: T) -> Self
    where
        T: Clone,
    {
        Self {
            start: StartBound(Included(value.clone())),
            end: EndBound(Included(value)),
        }
    }

    /// Get the start value of the range (if it is bounded)
    ///
    /// # Examples
    ///
    /// ```
    /// use rangemap::{Range, RangeBounds, Bound};
    ///
    /// let bounded = Range::new(5..10);
    /// assert_eq!(bounded.start_value(), Some(&5));
    ///
    /// let unbounded = Range::new(..10);
    /// assert_eq!(unbounded.start_value(), None);
    /// ```
    pub fn start_value(&self) -> Option<&T> {
        self.start.value()
    }

    /// Get the end value of the range (if it is bounded)
    ///
    /// # Examples
    ///
    /// ```
    /// use rangemap::{Range, RangeBounds, Bound};
    ///
    /// let bounded = Range::new(5..10);
    /// assert_eq!(bounded.end_value(), Some(&10));
    ///
    /// let unbounded = Range::new(5..);
    /// assert_eq!(unbounded.end_value(), None);
    /// ```
    pub fn end_value(&self) -> Option<&T> {
        self.end.value()
    }

    /// Shift the entire range by some value
    pub fn shift(&mut self, by: T)
    where
        T: Clone + core::ops::AddAssign,
    {
        if let Some(value) = self.start.value_mut() {
            *value += by.clone();
        }
        if let Some(value) = self.end.value_mut() {
            *value += by;
        }
    }

    /// Shift the entire range to the left by some value (useful if using an
    /// unsigned type, where [`shift`] isn't useful)
    pub fn shift_left(&mut self, by: T)
    where
        T: Clone + core::ops::SubAssign,
    {
        if let Some(value) = self.start.value_mut() {
            *value -= by.clone();
        }
        if let Some(value) = self.end.value_mut() {
            *value -= by;
        }
    }

    pub fn shift_right(&mut self, by: T)
    where
        T: Clone + core::ops::AddAssign,
    {
        if let Some(value) = self.start.value_mut() {
            *value += by.clone();
        }
        if let Some(value) = self.end.value_mut() {
            *value += by;
        }
    }

    // TODO
    // /// Adjust the start of a range to a new lower bound.
    // pub fn adjust_left(&mut self, _new_start: Bound<T>) -> Self {
    //     // TODO: panic if empty range
    //     todo!()
    // }

    // /// Adjust the end of a range to a new upper bound.
    // pub fn adjust_right(&mut self, _new_end: Bound<T>) -> Self {
    //     todo!()
    // }

    /// Converts from `&Range<T>` to `Range<&T>`.
    #[inline]
    pub fn as_ref(&self) -> Range<&T> {
        Range {
            start: self.start.as_ref(),
            end: self.end.as_ref(),
        }
    }

    pub(crate) fn bound_before(&self) -> Option<EndBound<&T>> {
        self.start.before()
    }
    // pub(crate) fn into_bound_after
    pub(crate) fn bound_after(&self) -> Option<StartBound<&T>> {
        self.end.after()
    }

    /// Check whether the range overlaps with another (i.e. the intersection is
    /// not null)
    pub fn overlaps(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        match (
            core::cmp::max(self.start.as_ref(), other.start.as_ref()).0,
            core::cmp::min(self.end.as_ref(), other.end.as_ref()).0,
        ) {
            // Both starts and/or both ends are unbounded, must overlap
            (Unbounded, _) | (_, Unbounded) => true,

            // If both are included, check less than or equal (overlap on same value)
            (Included(a), Included(b)) => a <= b,

            // Otherwise, no overlap on equal
            (Included(a) | Excluded(a), Included(b) | Excluded(b)) => a < b,
        }
    }

    /// Check whether the range touches another, either overlapping it or coming
    /// directly up to it (with no gap)
    ///
    /// See also: [`Range::overlaps`]
    pub fn touches(&self, other: &Self) -> bool
    where
        T: Ord,
    {
        match (
            core::cmp::max(self.start.as_ref(), other.start.as_ref()).0,
            core::cmp::min(self.end.as_ref(), other.end.as_ref()).0,
        ) {
            // Both starts and/or both ends are unbounded, must overlap
            (Unbounded, _) | (_, Unbounded) => true,

            // If both are excluded, ranges don't touch on equal
            // (there is a point gap when a==b)
            (Excluded(a), Excluded(b)) => a < b,

            // If either are included, check less than or equal
            // (overlap on both included, touching when only one is)
            (Included(a) | Excluded(a), Included(b) | Excluded(b)) => a <= b,
        }
    }

    pub fn up_to_and_including_start(&self) -> core::ops::RangeTo<Bound<&T>> {
        ..self.start.as_bound_inner_ref()
    }
}

impl<'a, T> Range<&'a T> {
    pub(crate) fn borrow_bound_before(&self) -> Option<EndBound<&'a T>> {
        self.start.borrow_before()
    }
    pub(crate) fn borrow_bound_after(&self) -> Option<StartBound<&'a T>> {
        self.end.borrow_after()
    }
}

// Utility, since it's messy to match everwhere
fn bound_cloned<T: Clone>(b: Bound<&T>) -> Bound<T> {
    match b {
        Unbounded => Unbounded,
        Included(x) => Included(x.clone()),
        Excluded(x) => Excluded(x.clone()),
    }
}
fn bound_value<T>(b: Bound<T>) -> Option<T> {
    match b {
        Unbounded => None,
        Included(t) => Some(t),
        Excluded(t) => Some(t),
    }
}

// TODO: add to above
// TODO: non-borrowed?
// impl<T> Range<&T> {
//     fn difference(&self, other: &Self) -> alloc::vec::Vec<Range<&T>> {

//         // If `range` is still fully before `other`, use it (and
//         // hold on to `other`)
//         if self.end.cmp_start(&other.start).is_gt() {
//             prev_other.insert(other);
//             return Some(range);
//         }

//         // If `range` is fully after `other`, grab the next
//         // `other` (and loop again)
//         if range.start.cmp_end(&other.end).is_gt() {
//             continue;
//         }

//         // Otherwise, we have some overlap
//         //
//         // The `borrow_bound_x().unwrap()` calls below should be
//         // fine, since they only occur where the match precludes
//         // an unbounded start/end.
//         match (range.start.cmp(&other.start), range.end.cmp(&other.end)) {
//             // Partial overlap, but `left` doesn't extend beyond `right`
//             // We can use part of `left` and forget the rest
//             (Less, Less) => {
//                 range.end = other.borrow_bound_before().unwrap();
//                 prev_other.insert(other);
//                 return Some(range);
//             }

//             // Partial overlap where `left` extends just to the
//             // end of `right` (don't save `right`)
//             (Less, Equal) => {
//                 range.end = other.borrow_bound_before().unwrap();
//                 return Some(range);
//             }

//             // Fully overlapped, but with some excess `right`
//             // Keep it and loop again with a new `left`.
//             (Greater | Equal, Less) => {
//                 range = self_iter.next()?.as_ref();
//                 prev_other.insert(other);
//             }

//             // Fully overlapped but with no more `right`, loop
//             // again with a new one of each
//             (Greater | Equal, Equal) => {
//                 range = self_iter.next()?.as_ref();
//                 continue;
//             }

//             // Partial overlap, but some `left` past `right`
//             // Keep part of `left` and look for a new `right`
//             (Greater | Equal, Greater) => {
//                 range.start = other.borrow_bound_after().unwrap();
//                 continue;
//             }

//             // `left` extends beyond `right` in both directions.
//             // Use the part of `left` before `right` and store
//             // the part after.
//             (Less, Greater) => {
//                 prev_self.insert(Range {
//                     start: other.borrow_bound_after().unwrap(),
//                     end: range.end.clone(),
//                 });
//                 range.end = other.borrow_bound_before().unwrap();
//                 return Some(range);
//             }
//         }
//     }
// }

impl<T: PartialEq, R: core::ops::RangeBounds<T>> PartialEq<R> for Range<T> {
    fn eq(&self, other: &R) -> bool {
        (match (&self.start.0, other.start_bound()) {
            (Unbounded, Unbounded) => true,
            (Included(t), Included(u)) | (Excluded(t), Excluded(u)) => t.eq(u),
            _ => false,
        }) && (match (&self.end.0, other.end_bound()) {
            (Unbounded, Unbounded) => true,
            (Included(t), Included(u)) | (Excluded(t), Excluded(u)) => t.eq(u),
            _ => false,
        })
    }
}

// impl<T> PartialEq<core::ops::RangeFull> for Range<T> {
//     fn eq(&self, _other: &core::ops::RangeFull) -> bool {
//         matches!(
//             (&self.start, &self.end),
//             (StartBound(Unbounded), EndBound(Unbounded)),
//         )
//     }
// }

// impl<T: Ord> PartialEq<core::ops::Range<T>> for Range<T> {
//     fn eq(&self, other: &core::ops::Range<T>) -> bool {
//         if let (StartBound(Included(start)), EndBound(Excluded(end))) = (&self.start, &self.end) {
//             other.start.eq(start) && other.end.eq(end)
//         } else {
//             false
//         }
//     }
// }

// impl<T: Ord> PartialEq<core::ops::RangeFrom<T>> for Range<T> {
//     fn eq(&self, other: &core::ops::RangeFrom<T>) -> bool {
//         if let (StartBound(Included(start)), EndBound(Unbounded)) = (&self.start, &self.end) {
//             other.start.eq(start)
//         } else {
//             false
//         }
//     }
// }

// impl<T: Ord> PartialEq<core::ops::RangeTo<T>> for Range<T> {
//     fn eq(&self, other: &core::ops::RangeTo<T>) -> bool {
//         if let (StartBound(Unbounded), EndBound(Excluded(end))) = (&self.start, &self.end) {
//             other.end.eq(end)
//         } else {
//             false
//         }
//     }
// }

// impl<T: Ord> PartialEq<core::ops::RangeInclusive<T>> for Range<T> {
//     fn eq(&self, other: &core::ops::RangeInclusive<T>) -> bool {
//         if let (StartBound(Included(start)), EndBound(Included(end))) = (&self.start, &self.end) {
//             other.start().eq(start) && other.end().eq(end)
//         } else {
//             false
//         }
//     }
// }

// impl<T: Ord> PartialEq<core::ops::RangeToInclusive<T>> for Range<T> {
//     fn eq(&self, other: &core::ops::RangeToInclusive<T>) -> bool {
//         if let (StartBound(Unbounded), EndBound(Included(end))) = (&self.start, &self.end) {
//             other.end.eq(end)
//         } else {
//             false
//         }
//     }
// }
