# A query string builder for percent encoding key-value pairs

[![Crates.io](https://img.shields.io/crates/v/query-string-builder)](https://crates.io/crates/query-string-builder)
[![Crates.io](https://img.shields.io/crates/l/query-string-builder)](https://crates.io/crates/query-string-builder)
[![codecov](https://codecov.io/gh/sunsided/query-string-builder/graph/badge.svg?token=HUCXM04DOG)](https://codecov.io/gh/sunsided/query-string-builder)

This is a tiny helper crate for simplifying the construction of URL query strings.
The initial `?` question mark is automatically prepended.

## Example

`QueryString` borrows its keys and values and allocates **no strings at all** —
neither while building nor while rendering. Only the pair list itself is a
single `Vec` allocation (see below to avoid even that):

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

## Owned builder

If you want to store the query string in a struct or return it from a function
without dealing with lifetimes, use `QueryStringOwned` — the same API, but it
percent-encodes keys and values eagerly into a single internal buffer. Pre-size
it with `with_capacity` to build with exactly one allocation, and take the
result with `as_str()` or `into_string()` without formatting or copying:

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

```rust,ignore
// Does not compile: the String temporary is dropped at the end of the statement.
let qs = QueryString::new().with("answer", &42.to_string());
println!("{qs}");
```

```rust,ignore
// Bind first instead:
let answer = 42.to_string();
let qs = QueryString::new().with("answer", &answer);
```
