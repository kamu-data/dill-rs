# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.3] - 2023-06-05
### Added
- Deduplicating errors in `ValidationError`
- `CatalogBuilder::validate()` now allows to specify types that are registered dynamically and should not be considered unresolved


## [0.5.2] - 2023-06-05
### Added
- Keeping a CHANGELOG
- `CatalogBuilder::validate()` function to validate the dependency graph
- `Maybe` spec for optional dependencies
- Derive macros now support injecting `Option<T>` (resolves to `Maybe<T>` spec) and `Vec<T>` (resolves to `AllOf<T>` spec)
