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

This crate is still in early stages and needs a lot of work, BUT it's [in active use in `kamu-cli`](https://github.com/kamu-data/kamu-cli/blob/601572a00702d15f630738b5fcad50ecafaed816/kamu-cli/src/app.rs#L89-L146) - a fairly large project organized according to [Onion/Clean Architecture](https://herbertograca.com/2017/11/16/explicit-architecture-01-ddd-hexagonal-onion-clean-cqrs-how-i-put-it-all-together/). We are continuing to improve this crate as we go and encounter more sophisticated DI scenarios.


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


# Features
- Injection specs:
  - `OneOf` - expects a single implementation of a given interface
  - `AllOf` - returns a collection of all implementations on a given interface
  - `Maybe<Spec>` - Returns `None` if inner `Spec` cannot be resolved
- Component scopes:
  - `Transient` (default) - a new instance is created for every invocation
  - `Singleton` - an instance is created upon first use and then reused for the rest of calls
- `#[component]` macro can derive `Builder`:
  - When used directly for a `struct` or on `impl` block with `Impl::new()` function
  - Can inject as `Arc<T>`, `T: Clone`, `&T`
  - `Option<T>` is interpreted as `OneOf<T>` spec
  - `Vec<T>` is interpreted as `AllOf<T>` spec
  - Supports custom argument bindings in `Builder`
- Prebuilt / add by value support
- By value injection of `Clone` types
- `Catalog` can be self-injected


# Design Principles
- **Non-intrusive**
  - Writing DI-friendly code should be as close as possible to writing regular types
  - DI should be an additive feature - we should be able to disable it and have all code compile (i.e. allowing for DI-optional libraries)
- **Externalizable**
  - It should be possible to add DI capabilities to 3rd party code
- Focus on **runtime** injection
  - Leveraging type system and zero-cost abstractions is great, but hard to get right - this project started because we needed something practical fast
  - Some cases involve dynamic registration of objects (e.g. adding an auth token during HTTP request processing), which further complicates compile-time DI
  - We use DI to integrate coarse-grained components, where some overhead is tolerable
  - We compensate for safety by providing runtime graph validation
- Focus on **constructor injection**
  - Field/property/accessor-based injection would complicate the system, and in our experience is of little use
- Put **implementation in control**
  - The type implementor (and not type user) usually has the best knowledge of what the optimal lifecycle for the type should be and its and concurrency characteristics, thus implementors should be in control of the defaults


# TODO
- Support `stable` rust
- Support builders providing own interface lists (default bindings)
- Improve graph validation
- Make `Scope`s external to `Builder`s so they could be overridden
- Allow dynamic registration (without cloning entire catalogs)
- Consider using traits to map `Arc`, `Option`, `Vec` to dependency specs instead of relying on macro magic
- Add `trybuild` tests (see https://youtu.be/geovSK3wMB8?t=956)
- Support generic types
- Replace `add_*` with generic `add<B: Into<Builder>>`
- value by reference in new()
- + Send + Sync plague  https://www.reddit.com/r/rust/comments/6dz0xh/abstracting_over_reference_counted_types_rc_and/
- Add low-overhead resolution stack to errors (e.g. populated on unwind)
- Extra scopes
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


