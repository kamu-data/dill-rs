<div align="center">
  <h1>dill</h1>
  <p>
    <strong>Runtime dependency injection library for Rust</strong>
  </p>
  <p>

[![Crates.io](https://img.shields.io/crates/v/dill.svg)](https://crates.io/crates/dill)
[![build](https://github.com/sergiimk/dill-rs/actions/workflows/build.yaml/badge.svg)](https://github.com/sergiimk/dill-rs/actions/workflows/build.yaml)

  </p>
</div>

# TODO
- Replace `add_*` with generic `add<B: Into<Builder>>`
- value by reference in new()
- + Send + Sync plague  https://www.reddit.com/r/rust/comments/6dz0xh/abstracting_over_reference_counted_types_rc_and/
- scopes
  - invocation
  - thread
  - task
  - catalog?
- thread safety
- adding values to catalog dynamically
- lazy values
- externally defined types
- custom builders
- error handling
- doctests
- Advanced queries (based on metadata + custom filters)
- improve catalog fluent interface (or macro?)
- proc macro error handling
- build a type without registering

# Done
- multiple implementations per interface
- implementation-controlled sharing and lifetime
- dependency specs
  - OneOf
  - AllOf
- scopes
  - transient
  - singleton
- auto builders
  - support scope in derivation
- support prebuilt / add by value
- support Impl::new()
- argument bindings
- by value injection of `Clone` types
- Separate catalog use from catalog building
- Make Catalog cloning cheap
- Catalog self-injection


# Principles
- Nothing framework-specific



- Create instance (ctor, new(), and external types)
- Provide dynamic dependencies -> recurse
- Provide fixed dependencies (Fn multi)
- Get existing instance if exists (scope)


- Separate builder from the scope, catalog, Arc stuff
- Pass build context instead of catalog (e.g. for stack tracking)