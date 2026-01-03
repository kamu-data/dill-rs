use std::sync::Arc;

use crate::injection_context::InjectionContext;
use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Catalog(pub(crate) Arc<CatalogImpl>);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl Catalog {
    /// Use [`Catalog::builder()`] to construct.
    pub(crate) fn new(pimpl: Arc<CatalogImpl>) -> Self {
        Self(pimpl)
    }

    /// Returns a [`CatalogBuilder`] used to initialize a [`Catalog`].
    pub fn builder() -> CatalogBuilder {
        CatalogBuilder::new()
    }

    /// Returns a [`CatalogBuilder`] that is chained to this [`Catalog`] and
    /// thus "inherits" all previous builders and bindings. Chaining catalogs is
    /// a very useful technique to add dynamic values that are not known
    /// upfront. For example, catalog chaining can be used in HTTP server
    /// middleware to add authorization information about the caller after
    /// validating the security token, or to open a database transaction and
    /// make it available for injection to all subsequently instantiated
    /// services.
    pub fn builder_chained(&self) -> CatalogBuilder {
        CatalogBuilder::new_chained(self)
    }

    /// Returns a weak reference to the catalog chain. Weak reference is useful
    /// when you want to keep using `Catalog` as a factory for complex
    /// instantiation logic, but don't want to own the strong reference that
    /// prevents it to be dropped. It's imperative to use weak references in
    /// [`Singleton`] and other caching scopes, as otherwise you may end up with
    /// cyclic references that prevent all instances from ever being cleaned up.
    pub fn weak_ref(&self) -> CatalogWeakRef {
        CatalogWeakRef::new(&self.0)
    }

    /// Returns an iterator over all registered instance [`Builder`]s.
    #[inline(always)]
    pub fn builders<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Builder> + 'a> {
        self.0.builders()
    }

    /// Returns an iterator over [`Builder`]s bound to a specific interface
    /// type.
    #[inline(always)]
    pub fn builders_for<'a, Iface>(
        &'a self,
    ) -> Box<dyn Iterator<Item = TypecastBuilder<'a, Iface>> + 'a>
    where
        Iface: 'static + ?Sized,
    {
        self.0.builders_for()
    }

    /// Filters [`Builder`]s by bound interface type and metadata predicate.
    #[inline(always)]
    pub fn builders_for_with_meta<'a, Iface, Meta>(
        &'a self,
        pred: impl Fn(&Meta) -> bool + Copy + 'a,
    ) -> Box<dyn Iterator<Item = TypecastBuilder<'a, Iface>> + 'a>
    where
        Iface: 'static + ?Sized,
        Meta: 'static,
    {
        self.0.builders_for_with_meta(pred)
    }

    /// Resolves and attempts to get an instance by a specific dependency
    /// [`DependencySpec`].
    #[inline(always)]
    pub fn get<Spec>(&self) -> Result<Spec::ReturnType, InjectionError>
    where
        Spec: DependencySpec + 'static,
    {
        self.get_with_context::<Spec>(&InjectionContext::new_root())
    }

    #[inline(always)]
    pub fn get_with_context<Spec>(
        &self,
        ctx: &InjectionContext,
    ) -> Result<Spec::ReturnType, InjectionError>
    where
        Spec: DependencySpec + 'static,
    {
        Spec::get(self, &ctx.push_resolve::<Spec>())
    }

    /// A short-hand for `get::<OneOf<T>>()`.
    #[inline(always)]
    pub fn get_one<Iface>(&self) -> Result<Arc<Iface>, InjectionError>
    where
        Iface: 'static + ?Sized + Send + Sync,
    {
        self.get::<OneOf<Iface>>()
    }

    /// Sets this catalog as "current" in the async task scope for the duration
    /// of the provided coroutine.
    ///
    /// Most useful when used in combination with [`Lazy`] and
    /// [`Self::builder_chained()`] for dynamically registering additional
    /// types.
    ///
    /// Scopes can be nested - at the end of the inner scope the catalog from an
    /// outer scope will be restored as "current".
    ///
    /// ### Examples
    ///
    /// ```
    /// use dill::*;
    /// use tokio::runtime::Runtime;
    ///
    /// Runtime::new().unwrap().block_on(async {
    ///     let cat = Catalog::builder().add_value(String::from("test")).build();
    ///
    ///     cat.scope(async move {
    ///         let val = Catalog::current().get_one::<String>().unwrap();
    ///         assert_eq!(val.as_str(), "test");
    ///     }).await;
    /// })
    /// ```
    #[cfg(feature = "tokio")]
    pub async fn scope<F, R>(&self, f: F) -> R
    where
        F: std::future::Future<Output = R>,
    {
        CURRENT_CATALOG.scope(self.clone(), f).await
    }

    /// Allows accessing the catalog in the current [`Self::scope`].
    ///
    /// Note that you should very rarely be using this method directly if at
    /// all. Instead, you should rely on [`Lazy`] for
    /// delayed injection from a current catalog.
    ///
    /// ### Panics
    ///
    /// Will panic if called from the outside of a [`Self::scope`].
    ///
    /// ### Examples
    ///
    /// ```
    /// use dill::*;
    /// use tokio::runtime::Runtime;
    ///
    /// Runtime::new().unwrap().block_on(async {
    ///     let cat = Catalog::builder().add_value(String::from("test")).build();
    ///
    ///     cat.scope(async move {
    ///         let val = Catalog::current().get_one::<String>().unwrap();
    ///         assert_eq!(val.as_str(), "test");
    ///     }).await;
    /// })
    /// ```
    #[cfg(feature = "tokio")]
    pub fn current() -> Catalog {
        CURRENT_CATALOG.get()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Debug for Catalog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Catalog(0x{:x})",
            self.0.as_ref() as *const CatalogImpl as usize
        )
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "tokio")]
tokio::task_local! {
    pub(crate) static CURRENT_CATALOG: Catalog;
}
