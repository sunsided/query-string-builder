# Changelog

All notable changes to this project will be documented in this file.
This project uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2024-05-23

[0.4.2]: https://github.com/sunsided/query-string-builder/releases/tag/v0.4.2

### Added

- More characters are added to the encoding set to ensure recursive values
  (e.g. URLs as a value) decode reliably.

### Fixed

- The hash character `#` is now encoded in order to ensure correct parsing of query parameters.

## [0.4.0] - 2023-07-08

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
