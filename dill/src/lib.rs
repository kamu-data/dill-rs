#![feature(unsize)]
#![feature(min_specialization)]
#![feature(error_generic_member_access)]

//! Runtime dependency injection.
//!
//! Documentation is under construction!
//!
//! # Examples
//!
//! ## Basic dependency resolution
//!
//! As a user of type `A` we only care about getting an instance to use - the
//! life-cycle of `A` and its dependencies remain hidden from us.
//!
//! ```
//! use dill::*;
//! use std::sync::Arc;
//!
//! #[component]
//! struct A {
//!     b: Arc<B>,  // An instance of `B` will be resolved and injected when requesting `A`
//! }
//!
//! impl A {
//!     fn foo(&self) -> String {
//!         format!("a::{}", self.b.bar())
//!     }
//! }
//!
//! #[component]
//! struct B;
//!
//! impl B {
//!     fn bar(&self) -> String {
//!         "b".to_string()
//!     }
//! }
//!
//! let catalog = Catalog::builder()
//!   .add::<A>()
//!   .add::<B>()
//!   .build();
//!
//! let a = catalog.get_one::<A>().unwrap();
//! assert_eq!(a.foo(), "a::b");
//! ```
//!
//! ## Using trait objects (aka Interfaces)
//!
//! Every type can be associated with multiple traits that it implements using
//! [`CatalogBuilder::bind()`] method, allowing dynamically picking the best
//! implementation to use (e.g. based on config) or even using multiple
//! implementations at once (e.g. plugins).
//!
//! ```
//! use dill::*;
//! use std::sync::Arc;
//!
//! // An interface that has two implementations below
//! trait A: Send + Sync {
//!     fn foo(&self) -> String;
//! }
//!
//! #[component]
//! struct AImpl1;
//! impl A for AImpl1 {
//!     fn foo(&self) -> String {
//!         "aimpl1".to_string()
//!     }
//! }
//!
//! #[component]
//! struct AImpl2;
//! impl A for AImpl2 {
//!     fn foo(&self) -> String {
//!         "aimpl2".to_string()
//!     }
//! }
//!
//! let catalog = Catalog::builder()
//!   .add::<AImpl1>()
//!   .bind::<dyn A, AImpl1>()
//!   .add::<AImpl2>()
//!   .bind::<dyn A, AImpl2>()
//!   .build();
//!
//! // AllOf<T> is a DependencySpec that returns instances of all types that implement trait T
//! let ays = catalog.get::<AllOf<dyn A>>().unwrap();
//!
//! let mut foos: Vec<_> = ays.iter().map(|a| a.foo()).collect();
//! foos.sort(); // Order is undefined
//!
//! assert_eq!(foos, vec!["aimpl1".to_owned(), "aimpl2".to_owned()]);
//! ```
//!
//! ## Controlling lifetimes with Scopes
//!
//! The life-cycle of a type is no longer controlled by the user of a type.
//! Author of type `A` below can choose whether `A` should be created per call
//! ([`Transient`]) or reused by all clients ([`Singleton`]).
//!
//! ```
//! use dill::*;
//!
//! #[component]
//! #[scope(Singleton)]
//! struct A {
//!     // Needed for compiler not to optimize type out
//!     name: String,
//! }
//!
//! impl A {
//!     fn test(&self) -> String {
//!         format!("a::{}", self.name)
//!     }
//! }
//!
//! let cat = Catalog::builder()
//!     .add::<A>()
//!     .add_value("foo".to_owned())
//!     .build();
//!
//! let inst1 = cat.get::<OneOf<A>>().unwrap();
//! let inst2 = cat.get::<OneOf<A>>().unwrap();
//!
//! // Expecting Singleton scope to return same instance
//! assert_eq!(
//!     inst1.as_ref() as *const A,
//!     inst2.as_ref() as *const A
//! );
//! ```
//!
//! ## Parametrizing builders
//!
//! Builders can be parametrized during the registration process for convenience
//! (e.g. with values read from configuration).
//!
//! ```
//! use dill::*;
//!
//! #[component]
//! #[scope(Singleton)]
//! struct ConnectionPool {
//!     host: String,
//!     port: i32,
//! }
//!
//! impl ConnectionPool {
//!     fn url(&self) -> String {
//!         format!("http://{}:{}", self.host, self.port)
//!     }
//! }
//!
//! let cat = Catalog::builder()
//!     .add_builder(
//!         ConnectionPool::builder()
//!             .with_host("foo".to_owned())
//!             .with_port(8080),
//!     )
//!     .build();
//!
//! let inst = cat.get::<OneOf<ConnectionPool>>().unwrap();
//! assert_eq!(inst.url(), "http://foo:8080");
//! ```

mod builder;
pub mod cache;
mod catalog;
mod catalog_builder;
mod errors;
mod lazy;
pub mod scopes;
pub mod specs;
mod typecast_builder;

pub use builder::*;
pub use catalog::Catalog;
pub use catalog_builder::CatalogBuilder;
pub use dill_impl::*;
pub use errors::*;
pub use lazy::Lazy;
pub use scopes::*;
pub use specs::*;
pub use typecast_builder::*;
