# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0] - 2025-03-28
### Changed
- BREAKING: `CatalogBuilder::add_builder()` method will automatically register interfaces (`#[interface(dyn Iface)]`) for `#[component]`.
  - If you were using `add_builder()` previously you'll need to check that you are not calling `bind()` for default interfaces as this will result in ambiguous dependency errors.
  - To disable auto-registration, a use:
    ```rust
    let catalog = CatalogBuilder::new()
        .add_builder(Impl::builder().without_default_interfaces())
        .build();
    ```

## [0.12.4] - 2025-03-26
### Fixed
- Suppress `clippy::too_many_arguments` warning in generated functions.

## [0.12.3] - 2025-03-26
### Changed
- The `#[component]` macro will generate `new()` factory function when applied to a `struct`. This prevents the situations when `impl` with `new` function is added, but developer forgets to move `#[component]` decorator from `struct` to `impl`, which can lead to confusion.
- The `#[component(no_new)]` attribute is added to skip generating `new` function.

## [0.12.2] - 2025-03-25
### Fixed
- Rename private `Builder::scope` field to avoid collisions with user-defined arguments.

## [0.12.1] - 2025-03-24
### Changed
- Simplified builder codegen for explicit arguments.
- Any `Clone` value can be used as explicit argument and will bypass all injection machinery.

## [0.12.0] - 2025-03-24
### Added
- `#[component(explicit)]` attribute allows to specify fields that must be passed during the builder construction instead of being injected.
- `TypedBuilderCast` trait allows to cast `TypedBuilder<T>` into `TypedBuilder<dyn I>` if `T` implements trait `I`.
- New `eaxmples/cli` example showcasing builder casting and explicit fields.
- Supported `Option<ByValue>` injection.
### Changed
- `Builder::get` was renamed to `Builder::get_any` to avoid the ambiguity between it and `TypedBuilder::get`.

## [0.11.0] - 2025-01-15
### Changed
- Upgraded to latest nightly Rust compiler and deps

## [0.10.0] - 2024-12-09
### Added
- `Catalog::scope` and `Catalog::current` allow setting and accessing a "current" catalog in a task-local context
  - Requires new `tokio` crate feature
- `Lazy<T>` injection spec that delays the creation of a value until it's requested
  - Can be used to delay initialization of expensive values that are rarely used
  - Can be used in combination with `Catalog::scope` to inject values registered dynamically

## [0.9.3] - 2024-12-06
### Changed
- Upgraded to `thiserror v2` dependency

## [0.9.2] - 2024-10-02
### Added
- `Catalog::builder()` and `catalog.builder_chained()` shortcuts
- New `examples` directory to showcase DI patterns and integrations

## [0.9.1] - 2024-08-15
### Fixed
- `Catalog::builders_for_with_meta()` works correctly for chained catalogs

## [0.9.0] - 2024-07-29
### Added
- It's now possible to associate custom static metadata with builders:
  ```rust
  #[component]
  #[interface(dyn EventHandler)]
  #[meta(EventHandlerDesc { event_type: "A"})]
  #[meta(EventHandlerDesc { event_type: "B"})]
  struct EventHandlerAB;
  ```
- New `BuilderExt` trait was added to provide convenient access to metadata and interfaces
- New `Catalog::builders_for_with_meta()` allows to filter builders by metadata with a custom predicate
### Changed
- `Builder::interfaces` method was changed to iteration via callback to avoid allocating a `Vec`

## [0.8.1] - 2024-05-27
### Fixed
- Fixed pedantic linter warnings

## [0.8.0] - 2023-11-27
### Added
- Added `interface(T)` macro attribute to provide default interface bindings for the component builders, allowing to cut down verboseness of catalog configuration. Example:
  ```rust
  trait Iface {}

  #[dill::component]
  #[dill::interface(dyn Iface)]
  struct Impl;

  // Automatically does `.bind::<dyn Iface, Impl>()`
  Catalog::builder().add::<Impl>().build();
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
