use std::sync::Arc;

use crate::InjectionError;

/// Represents a value whose construction is delayed upon request rather than
/// resolved from the catalog immediately. This is often useful in cases when
/// some expensive type is used rarely, thus it's beneficial to only construct
/// it on-demand.
///
/// When instance is requested, this type will first attempt to resolve it using
/// [`crate::Catalog::current`] and will fall back to the catalog instance used
/// to create the `Lazy<T>`. This means that `Lazy<T>` can be used to access
/// values that are dynamically added into a chain of catalogs and may not be
/// present when the `Lazy<T>` itself is injected.
///
/// Note that this style of injection still respects the component's control
/// over its own lifetime via [`crate::Scope`]. So It's recommended to use
/// [`Self::get`] liberally and release the instances ASAP without attempting
/// to cache them.
///
/// See also:
/// - [`crate::Catalog::builder_chained`]
/// - [`crate::Catalog::scope`]
/// - [`crate::Catalog::current`]
///
/// ### Examples
///
/// ```
/// #[dill::component]
/// struct A;
///
/// impl A {
///     fn test(&self) -> String {
///         "A".into()
///     }
/// }
///
/// #[dill::component]
/// struct B {
///     lazy_a: dill::Lazy<std::sync::Arc<A>>,
/// }
/// impl B {
///     fn test(&self) -> String {
///         // A is created on-demand during this call
///         let a = self.lazy_a.get().unwrap();
///         a.test()
///     }
/// }
///
/// let cat = dill::Catalog::builder()
///     .add::<A>()
///     .add::<B>()
///     .build();
/// ```
#[derive(Clone)]
pub struct Lazy<T> {
    factory: Arc<dyn Fn() -> Result<T, InjectionError> + Send + Sync>,
}

impl<T> std::fmt::Debug for Lazy<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lazy").finish_non_exhaustive()
    }
}

impl<T> Lazy<T> {
    pub fn new(f: impl Fn() -> Result<T, InjectionError> + Send + Sync + 'static) -> Self {
        Self {
            factory: Arc::new(f),
        }
    }

    pub fn get(&self) -> Result<T, InjectionError> {
        (self.factory)()
    }
}
