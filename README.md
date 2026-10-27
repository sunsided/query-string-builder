# A query string builder for percent encoding key-value pairs

[![Crates.io](https://img.shields.io/crates/v/query-string-builder)](https://crates.io/crates/query-string-builder)
[![Crates.io](https://img.shields.io/crates/l/query-string-builder)](https://crates.io/crates/query-string-builder)
[![codecov](https://codecov.io/gh/sunsided/query-string-builder/graph/badge.svg?token=HUCXM04DOG)](https://codecov.io/gh/sunsided/query-string-builder)

This is a tiny helper crate for simplifying the construction of URL query strings.
The initial `?` question mark is automatically prepended.

## Example

`QueryString` borrows its keys and values and allocates **no strings at all** —
neither while building nor while rendering. The only heap memory is the `Vec`
holding the pairs; size it up front with `QueryString::with_capacity(n)` to
make that a single allocation, or see below to avoid the heap entirely:

```rust
use query_string_builder::QueryString;

fn main() {
    let tasty = true;
    let weight = 70.0;

    let qs = QueryString::new()
        .with("q", "apple")
        .with("tasty", &tasty)
        .with("weight", &weight)
        .with_opt("color", None::<&str>)
        .with_opt("category", Some("fruits and vegetables?"));

    assert_eq!(
        format!("https://example.com/{qs}"),
        "https://example.com/?q=apple&tasty=true&weight=70&category=fruits%20and%20vegetables?"
    );
}
```

To append the query string to an existing URL with a single, exactly sized
reservation (instead of the repeated growth of `format!`), use `write_into`;
`encoded_len` gives the exact length without allocating:

```rust
use query_string_builder::QueryString;

let mut url = String::from("https://example.com/");
let qs = QueryString::new().with("q", "apple");
assert_eq!(qs.encoded_len(), 8);
qs.write_into(&mut url);
assert_eq!(url, "https://example.com/?q=apple");
```

## No heap at all

The borrowing builder can keep its pairs inline instead of in a `Vec`.
`QueryString::inline::<N>()` holds up to `N` pairs without touching the heap;
`try_with`/`try_push` report a full builder instead of panicking:

```rust
use query_string_builder::QueryString;

let answer = 42;
let qs = QueryString::inline::<2>()
    .with("q", "apple")
    .with("answer", &answer);

assert_eq!(qs.to_string(), "?q=apple&answer=42");
```

Any other collection — e.g. `heapless::Vec` or `arrayvec::ArrayVec` — can be
plugged in by implementing `PairStorage` and passing it to
`QueryString::with_storage`.

## `no_std`

The crate is `#![no_std]`. Its default `alloc` feature provides
`QueryStringOwned`, the `Vec`-backed storage and the `String` helpers
(`into_owned`, `write_into`); disable default features to use it on targets
without an allocator, building on `InlinePairs` and rendering into any
`core::fmt::Write` sink:

```toml
query-string-builder = { version = "0.8", default-features = false }
```

```rust
use core::fmt::{self, Write};
use query_string_builder::{InlinePairs, QueryString};

fn write_search_path(out: &mut impl Write) -> fmt::Result {
    let qs = QueryString::with_storage(InlinePairs::<2>::new())
        .with("q", "apple")
        .with("color", "red");
    write!(out, "/search{qs}")
}
# let mut out = String::new();
# write_search_path(&mut out).unwrap();
# assert_eq!(out, "/search?q=apple&color=red");
```

## Owned builder

If you want to store the query string in a struct or return it from a function
without dealing with lifetimes, use `QueryStringOwned`. It shares the
`with`/`with_opt`/`push`/`push_opt`/`append` methods but takes keys and values
by value and percent-encodes them eagerly into a single internal buffer.
Pre-size it with `with_capacity` to build with exactly one allocation, and take
the result with `as_str()` or `into_string()` without formatting or copying:

```rust
use query_string_builder::QueryStringOwned;

fn build_query() -> QueryStringOwned {
    QueryStringOwned::new()
        .with("q", "apple")
        .with("answer", 42)
}
```

```rust
use query_string_builder::QueryStringOwned;

let url = QueryStringOwned::with_capacity(64)
    .with("q", "apple")
    .with("answer", 42)
    .into_string();
assert_eq!(url, "?q=apple&answer=42");
```

A borrowing builder can also be converted with `qs.into_owned()`.

## Borrowing footgun

Because `QueryString` borrows its values, inline temporaries don't live long
enough — bind them to a variable first:

```rust,compile_fail
use query_string_builder::QueryString;

// Does not compile: the String temporary is dropped at the end of the statement.
let qs = QueryString::new().with("answer", &42.to_string());
println!("{qs}");
```

```rust
use query_string_builder::QueryString;

// Bind first instead:
let answer = 42.to_string();
let qs = QueryString::new().with("answer", &answer);
assert_eq!(qs.to_string(), "?answer=42");
```
