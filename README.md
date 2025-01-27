# [`segmap`]

<!-- [![Crate](https://img.shields.io/crates/v/segmap.svg)](https://crates.io/crates/segmap)
[![Docs](https://docs.rs/segmap/badge.svg)](https://docs.rs/segmap)
[![Build status](https://github.com/ecclarke42/segmap/workflows/CI/badge.svg)](https://github.com/ecclarke42/segmap/actions)
[![Rust](https://img.shields.io/badge/rust-1.43%2B-blue.svg?maxAge=3600)](https://github.com/ecclarke42/segmap) -->
<!-- Don't forget to update the GitHub actions config when bumping minimum Rust version. -->

`segmap` exposes `SegmentMap`, a map data structure whose keys are stored as
ranges. Contiguous and overlapping ranges that map to the same value are
coalesced into a single range.

A correspoinding `SegmentSet` structure is also provided.

## The `Segment<T>` Type

`SegmentMap` supports all types of input range types in the same map and coerces
them all to a common range type for internal representation. A `Segment<T>` is
always represented as increasing, so "backwards" ranges will be flipped for
insertion.

Most methods on `SegmentMap` and `SegmentSet` accept a generic argument for the
range, which only needs to implement `RangeBounds`.

## Example: use with Chrono

```rust
use chrono::offset::TimeZone;
use chrono::{Duration, Utc};
use segmap::SegmentMap;

let people = ["Alice", "Bob", "Carol"];
let mut roster = SegmentMap::new();

// Set up initial roster.
let start_of_roster = Utc.ymd(2019, 1, 7);
let mut week_start = start_of_roster;
for _ in 0..3 {
    for person in people {
        let next_week = week_start + Duration::weeks(1);
        roster.insert(week_start..next_week, person);
        week_start = next_week;
    }
}

// Bob is covering Alice's second shift (the fourth shift overall).
let fourth_shift_start = start_of_roster + Duration::weeks(3);
let fourth_shift_end = fourth_shift_start + Duration::weeks(1);
roster.insert(fourth_shift_start..fourth_shift_end, "Bob");

// Print out the roster, and observe that
// the fourth and fifth shifts have been coalesced
// into one range.
for (range, &person) in roster.iter() {
    let start = *range.start_value().unwrap();
    let duration = *range.end_value().unwrap() - start;
    println!("{} ({}): {}", start, duration, person);
}

// Output:
// 2019-01-07UTC (P7D): Alice
// 2019-01-14UTC (P7D): Bob
// 2019-01-21UTC (P7D): Carol
// 2019-01-28UTC (P14D): Bob
// 2019-02-11UTC (P7D): Carol
// 2019-02-18UTC (P7D): Alice
// 2019-02-25UTC (P7D): Bob
// 2019-03-04UTC (P7D): Carol
```

## Building without the Rust standard library

This crate can work without the full standard library available
(e.g. when running on bare metal without an operating system)
but relies on the presence of a global allocator &mdash;
i.e. it links the `core` and `alloc` crates, but not `std`.

Presently there is no functionality in the crate that require
the standard library. Such functionality will likely be
introduced in the future, and will be gated behind a default-on
`std` feature.

See [The Rust Programming Language](https://doc.rust-lang.org/1.7.0/book/no-stdlib.html)
book for general information about operating without the standard library.

[`segmap`]: https://docs.rs/segmap/latest/segmap/struct.SegmentMap.html
