# A query string builder for percent encoding key-value pairs

[![Crates.io](https://img.shields.io/crates/v/query-string-builder)](https://crates.io/crates/query-string-builder)
[![Crates.io](https://img.shields.io/crates/l/query-string-builder)](https://crates.io/crates/query-string-builder)
[![codecov](https://codecov.io/gh/sunsided/query-string-builder/graph/badge.svg?token=HUCXM04DOG)](https://codecov.io/gh/sunsided/query-string-builder)

This is a tiny helper crate for simplifying the construction of URL query strings.
The initial `?` question mark is automatically prepended.

## Example

```rust
use query_string_builder::QueryString;

fn main() {
    let qs = QueryString::new()
        .with_value("q", "apple")
        .with_value("tasty", true)
        .with_opt_value("color", None::<String>)
        .with_opt_value("category", Some("fruits and vegetables?"));

    assert_eq!(
        format!("https://example.com/{qs}"),
        "https://example.com/?q=apple&tasty=true&category=fruits%20and%20vegetables?&tasty=true"
    );
}
```
