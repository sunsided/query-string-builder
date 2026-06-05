# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1] - 2026-06-05

[0.7.1]: https://github.com/sunsided/query-string-builder/releases/tag/v0.7.1

### Changed

- The crate now uses Rust edition 2024; the MSRV is raised to 1.87.
- Dependencies were upgraded (`percent-encoding` 2.3.2).

## [0.7.0] - 2026-06-05

[0.7.0]: https://github.com/sunsided/query-string-builder/releases/tag/v0.7.0

### Changed (breaking)

- `QueryString` is now a zero-allocation borrowing builder `QueryString<'a>`:
  it borrows keys and values (any `Display` type) and allocates no strings,
  neither while building nor while rendering — only a single `Vec` for the
  pair list. Construct it with `QueryString::new()`.
- The builder methods were shortened: `with_value` → `with` and
  `with_opt_value` → `with_opt` (doc aliases preserve searchability).
- The `simple()` and `dynamic()` constructors were removed.

### Added

- `QueryStringOwned`: the owning twin of `QueryString` with the identical API
  and no lifetime parameter, for storing the builder in structs or returning
  it from functions (eagerly allocates via `ToString`, like the 0.6
  `QueryString`).
- `QueryString::into_owned()` converts a borrowing builder into a
  `QueryStringOwned`, rendering each pair to owned strings.
- `IntoPart` / `Part`: the conversion trait and storage type backing the
  borrowing builder's arguments (`&str`, `&String`, `&i32`, `&bool`, …).
- `rust-version = "1.66"` (MSRV) is now declared in `Cargo.toml`.

### Removed

- `QueryStringSimple` and `WrappedQueryString` (the nested-generics slim
  builder) were removed; `QueryString<'a>` supersedes them with a plain,
  storable type and truly allocation-free rendering.
- The borrowing `QueryString` no longer implements `PartialEq`/`Eq`
  (its `Debug` implementation is hand-written and renders the pairs).

### Migration

| 0.6                                     | 0.7                                          |
|-----------------------------------------|----------------------------------------------|
| `QueryString::dynamic()`                | `QueryStringOwned::new()` (same behavior)    |
| `QueryString::simple()`                 | `QueryString::new()` (now truly zero-alloc)  |
| `.with_value(k, v)`                     | `.with(k, &v)` / owned: `.with(k, v)`        |
| `.with_opt_value(k, v)`                 | `.with_opt(k, v.as_ref())` / owned: as-is    |
| `QueryStringSimple`/`WrappedQueryString`| removed; use `QueryString<'a>`               |

Note: the borrowing `QueryString` borrows its values — bind temporaries to a
variable before passing them (`let v = x.to_string();` then `.with("k", &v)`).

## [0.6.0] - 2024-06-08

[0.6.0]: https://github.com/sunsided/query-string-builder/releases/tag/v0.6.0

### Added

- The `QueryString::simple` function was added to construct the new `QueryStringSimple` type.
  This type reduces string allocations, defers rendering and can keep references
  but at the cost of a complex type signature slightly more rigid handling.

### Changed

- The `QueryString::new` function was renamed to `QueryString::dynamic`.

## [0.5.1] - 2024-05-24

[0.5.1]: https://github.com/sunsided/query-string-builder/releases/tag/v0.5.1

### Internal

- `write_char()` and `fmt()` calls are now used instead of `write!` when rendering the string.

## [0.5.0] - 2024-05-24

[0.5.0]: https://github.com/sunsided/query-string-builder/releases/tag/v0.5.0

### Changed

- [#3](https://github.com/sunsided/query-string-builder/pull/3):
  The functions now change inputs that implement `ToString` rather than requiring `Into<String>`.
  This allows for any `Display` types to be used directly.

## [0.4.2] - 2024-05-23

[0.4.2]: https://github.com/sunsided/query-string-builder/releases/tag/v0.4.2

### Added

- More characters are added to the encoding set to ensure recursive values
  (e.g. URLs as a value) decode reliably.

### Fixed

- The hash character `#` is now encoded in order to ensure correct parsing of query parameters.

## [0.4.1] - 2023-07-08

[0.4.1]: https://github.com/sunsided/query-string-builder/releases/tag/0.4.1

### Internal

- The license `EUPL-1.2` is now explicitly specified in `Cargo.toml`, allowing it to show up correctly on crates.io.

## [0.4.0] - 2023-07-08

[0.4.0]: https://github.com/sunsided/query-string-builder/releases/tag/0.4.0

### Added

- The `QueryBuilder` now owns all string values, making it easier to pass
  a `QueryBuilder` value out of a function.

## [0.3.0] - 2023-07-08

### Added

- Added the `append` and `append_into` functions.

## [0.2.0] - 2023-07-07

### Added

- Added the `with_opt_value` and `push_opt` functions.

## [0.1.0] - 2023-07-07

### Internal

- 🎉 Initial release.

[0.3.0]: https://github.com/sunsided/query-string-builder/releases/tag/0.3.0

[0.2.0]: https://github.com/sunsided/query-string-builder/releases/tag/0.2.0

[0.1.0]: https://github.com/sunsided/query-string-builder/releases/tag/0.1.0
