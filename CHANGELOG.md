# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1] - 2024-05-27
### Changed
- `CatalogBuilder::new_chained()` gets the `chained_catalog` argument by value, not by reference

## [0.8.0] - 2023-11-27
### Added
- Added `interface(T)` macro attribute to provide default interface bindings for the component builders, allowing to cut down verboseness of catalog configuration. Example:
  ```rust
  trait Iface {}

  #[dill::component]
  #[dill::interface(dyn Iface)]
  struct Impl;

  // Automatically does `.bind::<dyn Iface, Impl>()`
  CatalogBuilder::new().add::<Impl>().build();
  ```

## [0.7.2] - 2023-09-04
### Changed
- Fixed validation of Catalog self-injection

## [0.7.1] - 2023-08-30
### Changed
- Linked with latest dependencies

## [0.7.0] - 2023-08-30
### Added
- Basic support for chaining catalogs

## [0.6.1] - 2023-06-15
### Fixed
- Account for overridden fields in generated builders when validating dependencies

## [0.6.0] - 2023-06-05
### Added
- `CatalogBuilder::add_builder()` now accepts `Arc<T>` and `Fn() -> Arc<T>`
- `CatalogBuilder::add_factory()` was replaced by `CatalogBuilder::add_value_lazy()` that accepts `FnOnce() -> T` and will cache result forever

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
