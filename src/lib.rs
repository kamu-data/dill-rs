#![feature(unsize)]

//! Runtime dependency injection.
//!
//! Documentation is under construction.

pub use dill_impl::*;

mod builder;
pub use builder::*;

mod catalog;
pub use catalog::*;

mod errors;
pub use errors::*;

mod specs;
pub use specs::*;

mod scopes;
pub use scopes::*;
