<div align="center">
  <h1>dill</h1>
  <p>
    <strong>Runtime dependency injection library for Rust</strong>
  </p>
  <p>

[![Crates.io](https://img.shields.io/crates/v/dill.svg?style=for-the-badge)](https://crates.io/crates/dill)
[![CI](https://img.shields.io/github/actions/workflow/status/sergiimk/dill-rs/build.yaml?logo=githubactions&label=CI&logoColor=white&style=for-the-badge&branch=master)](https://github.com/sergiimk/dill-rs/actions)
[![Dependencies](https://deps.rs/repo/github/sergiimk/dill-rs/status.svg?&style=for-the-badge)](https://deps.rs/repo/github/sergiimk/dill-rs)

  </p>
</div>

# Example

```rust
/////////////////////////////////////////

// Define interfaces in traits
trait A: Send + Sync {
    fn test(&self) -> String;
}

// Implement traits to define components
#[component]
struct AImpl {
    // Auto-inject dependencies (also supports by-value)
    b: Arc<dyn B>,
}

impl A for AImpl {
    fn test(&self) -> String {
        format!("aimpl::{}", self.b.test())
    }
}

/////////////////////////////////////////

trait B: Send + Sync {
    fn test(&self) -> String;
}

#[component]
struct BImpl;

impl B for BImpl {
    fn test(&self) -> String {
        "bimpl".to_owned()
    }
}

/////////////////////////////////////////

// Register interfaces and bind them to implementations
let cat = CatalogBuilder::new()
    .add::<AImpl>()
    .bind::<dyn A, AImpl>()
    .add::<BImpl>()
    .bind::<dyn B, BImpl>()
    .build();

// Get objects and have their deps satisfied automatically
let inst = cat.get::<OneOf<dyn A>>().unwrap();
assert_eq!(inst.test(), "aimpl::bimpl");
```

# TODO
- Add `trybuild` tests (see https://youtu.be/geovSK3wMB8?t=956)
- Support generic types
- Replace `add_*` with generic `add<B: Into<Builder>>`
- value by reference in new()
- + Send + Sync plague  https://www.reddit.com/r/rust/comments/6dz0xh/abstracting_over_reference_counted_types_rc_and/
- scopes
  - invocation
  - thread
  - task
  - catalog?
- optional / default dependencies
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