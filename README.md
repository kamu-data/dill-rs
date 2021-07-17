<div align="center">
  <h1>dill</h1>
  <p>
    <strong>Runtime dependency injection library for Rust</strong>
  </p>
  <p>

[![build](https://github.com/sergiimk/dill-rs/build/badge.svg)](https://github.com/sergiimk/dill-rs/actions)

  </p>
</div>

# TODO
- value by reference in new()
- + Send + Sync plague  https://www.reddit.com/r/rust/comments/6dz0xh/abstracting_over_reference_counted_types_rc_and/
- scopes
  - invocation
  - thread
  - task
- thread safety
- catalog cloning
  - dynamic values
- lazy values
- externally defined types
- custom builders
- error handling
- doctests
- by value (cloneable?) injection
- metadata + filtering
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
  - transient  V
  - singleton  V
- auto builders
  - support scope in derivation
- support prebuilt / add by value
- support Impl::new()
- argument bindings


# Principles
- Nothing framework-specific