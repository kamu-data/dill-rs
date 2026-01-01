use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use multimap::MultiMap;

use crate::injection_context::InjectionContext;
use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct IfaceTypeId(pub TypeId);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct ImplTypeId(pub TypeId);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Catalog(pub(crate) Arc<CatalogInner>);

#[derive(Clone)]
pub(crate) struct CatalogInner {
    pub(crate) builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    pub(crate) bindings: MultiMap<IfaceTypeId, Binding>,
    pub(crate) chained_catalog: Option<Catalog>,
}

impl Catalog {
    pub fn builder() -> CatalogBuilder {
        CatalogBuilder::new()
    }

    pub fn builder_chained(&self) -> CatalogBuilder {
        CatalogBuilder::new_chained(self)
    }

    pub(crate) fn new(
        builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
        bindings: MultiMap<IfaceTypeId, Binding>,
        chained_catalog: Option<Catalog>,
    ) -> Self {
        Self(Arc::new(CatalogInner {
            builders,
            bindings,
            chained_catalog,
        }))
    }

    pub fn builders<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Builder> + 'a> {
        let it_builders = self.0.builders.values().map(|b| b.as_ref());
        if let Some(chained_catalog) = &self.0.chained_catalog {
            Box::new(it_builders.chain(chained_catalog.builders()))
        } else {
            Box::new(it_builders)
        }
    }

    pub fn builders_for<'a, Iface>(
        &'a self,
    ) -> Box<dyn Iterator<Item = TypecastBuilder<'a, Iface>> + 'a>
    where
        Iface: 'static + ?Sized,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let bindings = self.0.bindings.get_vec(&iface_type);
        let it_bindings = TypecastBuilderIterator::new(bindings);

        if let Some(chained_catalog) = &self.0.chained_catalog {
            Box::new(it_bindings.chain(chained_catalog.builders_for::<Iface>()))
        } else {
            Box::new(it_bindings)
        }
    }

    pub fn builders_for_with_meta<'a, Iface, Meta>(
        &'a self,
        pred: impl Fn(&Meta) -> bool + Copy + 'a,
    ) -> Box<dyn Iterator<Item = TypecastBuilder<'a, Iface>> + 'a>
    where
        Iface: 'static + ?Sized,
        Meta: 'static,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let bindings = self.0.bindings.get_vec(&iface_type);

        let it_bindings =
            TypecastPredicateBuilderIterator::new(bindings, move |b| b.metadata_contains(pred));

        if let Some(chained_catalog) = &self.0.chained_catalog {
            Box::new(it_bindings.chain(chained_catalog.builders_for_with_meta::<Iface, Meta>(pred)))
        } else {
            Box::new(it_bindings)
        }
    }

    #[inline]
    pub fn get<Spec>(&self) -> Result<Spec::ReturnType, InjectionError>
    where
        Spec: DependencySpec + 'static,
    {
        self.get_with_context::<Spec>(&InjectionContext::new_root())
    }

    #[inline]
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
    #[inline]
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

#[cfg(feature = "tokio")]
tokio::task_local! {
    pub(crate) static CURRENT_CATALOG: Catalog;
}
