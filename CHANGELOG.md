# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/AlixANNERAUD/embedded_i18n/compare/v0.1.3...v0.2.0) - 2026-11-09

### Added

- *(macros)* building blocks for the rust-i18n style API ([#63](https://github.com/AlixANNERAUD/embedded_i18n/pull/63))
- [**breaking**] rust-i18n style API ([#66](https://github.com/AlixANNERAUD/embedded_i18n/pull/66))

### Fixed

- point CI, badges and the manifest to the renamed repository ([#65](https://github.com/AlixANNERAUD/embedded_i18n/pull/65))

### Miscellaneous

- set up mise and Mr Boxington ([#58](https://github.com/AlixANNERAUD/embedded_i18n/pull/58))
- bump the github-actions group across 1 directory with 2 updates ([#61](https://github.com/AlixANNERAUD/embedded_i18n/pull/61))
- *(deps)* bump the cargo group across 1 directory with 5 updates ([#64](https://github.com/AlixANNERAUD/embedded_i18n/pull/64))

## [0.1.3](https://github.com/AlixANNERAUD/embedded-i18n/compare/v0.1.2...v0.1.3) - 2026-10-19

### Fixed

- support pt-BR in Unicode font range lookup ([#56](https://github.com/AlixANNERAUD/embedded-i18n/pull/56))

### Other

- Serialize environment-dependent tests with a shared lock

## [0.1.2](https://github.com/AlixANNERAUD/embedded-i18n/compare/v0.1.1...v0.1.2) - 2026-10-13

### Documentation

- add per-topic guide pages and document the macros crate ([#54](https://github.com/AlixANNERAUD/embedded-i18n/pull/54))

## [0.1.1](https://github.com/AlixANNERAUD/embedded-i18n/compare/v0.1.0...v0.1.1) - 2026-10-13

### Fixed

- include the last code point of each Unicode block range ([#51](https://github.com/AlixANNERAUD/embedded-i18n/pull/51))

## [0.1.0](https://github.com/AlixANNERAUD/embedded-i18n/releases/tag/v0.1.0) - 2026-10-13

### Added

- gate allocating helpers behind an alloc feature ([#32](https://github.com/AlixANNERAUD/embedded-i18n/pull/32))
- add allocation-free interpolation macros ([#33](https://github.com/AlixANNERAUD/embedded-i18n/pull/33))
- make the locales directory configurable ([#34](https://github.com/AlixANNERAUD/embedded-i18n/pull/34))
- support locale fallback chains ([#35](https://github.com/AlixANNERAUD/embedded-i18n/pull/35))
- support nested keys and YAML, TOML, MO and Fluent locale files ([#36](https://github.com/AlixANNERAUD/embedded-i18n/pull/36))
- support message context ([#37](https://github.com/AlixANNERAUD/embedded-i18n/pull/37))
- support plural forms ([#38](https://github.com/AlixANNERAUD/embedded-i18n/pull/38))
- support select variants ([#39](https://github.com/AlixANNERAUD/embedded-i18n/pull/39))
- add strict mode for translation coverage ([#40](https://github.com/AlixANNERAUD/embedded-i18n/pull/40))
- add bidi isolation and pseudo-localization ([#41](https://github.com/AlixANNERAUD/embedded-i18n/pull/41))
- localize date and time formatting without allocation ([#42](https://github.com/AlixANNERAUD/embedded-i18n/pull/42))
- embed several locales with runtime selection ([#43](https://github.com/AlixANNERAUD/embedded-i18n/pull/43))

### Documentation

- mention PO files in the README introduction ([#28](https://github.com/AlixANNERAUD/embedded-i18n/pull/28))
- add crate-level and item documentation ([#44](https://github.com/AlixANNERAUD/embedded-i18n/pull/44))
- rewrite README for discoverability ([#45](https://github.com/AlixANNERAUD/embedded-i18n/pull/45))

### Fixed

- rebuild when locale files change ([#26](https://github.com/AlixANNERAUD/embedded-i18n/pull/26))
- *(po)* ignore fuzzy entries ([#27](https://github.com/AlixANNERAUD/embedded-i18n/pull/27))

### Miscellaneous

- automate releases with release-plz ([#2](https://github.com/AlixANNERAUD/embedded-i18n/pull/2))
- remove unused dependencies from embedded-i18n-macros ([#24](https://github.com/AlixANNERAUD/embedded-i18n/pull/24))
- check no_std builds and MSRV ([#29](https://github.com/AlixANNERAUD/embedded-i18n/pull/29))
- enable Dependabot for Cargo and GitHub Actions ([#31](https://github.com/AlixANNERAUD/embedded-i18n/pull/31))

### Other

- Initial commit
- Initialize project with Cargo configuration and basic addition function
- Implement compile-time internationalization with macros and locale support
- Add English and French locale files, implement tests for locale functionality
- Reorganize package metadata in Cargo.toml for clarity and consistency
- Add GitHub Actions workflow for CI checks and coverage reporting
- Fix test for merging contiguous ranges to use Range struct
- Update README.md for improved installation instructions and API reference
- Fix formatting and punctuation in README.md for clarity
- Update license from GPL-2.0-only to MIT in Cargo.toml and LICENSE files
- Add support for JSON and PO backend localization with new features and dependencies
- Update README.md to include support for PO files and clarify backend selection
- Fix formatting in README.md for consistency in section headings and clarify backend behavior
