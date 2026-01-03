use std::sync::{Arc, Weak};

use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A weak reference to the `Catalog` that can be safely injected into the
/// components and used as a factory. While generally considered an
/// anti-pattern, this provides a great flexibility over when and how instances
/// are constructed.
///
/// When used with chained catalogs, weak references will be stored to all
/// catalogs in the chain and the first catalog that is still alive will be used
/// for instantiation. It is up to you to keep the catalog instances alive.
#[derive(Clone)]
pub struct CatalogWeakRef(Arc<[std::sync::Weak<CatalogImpl>]>);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl CatalogWeakRef {
    pub(crate) fn new(pimpl: &std::sync::Arc<CatalogImpl>) -> Self {
        // We keep weak references to all catalogs in the chain and will use the first
        // one that is still alive

        // Count depth
        let mut len = 0;
        let mut current = Some(pimpl);
        while let Some(p) = current {
            len += 1;
            current = p.chained_catalog.as_ref();
        }

        // Fill buffer
        let mut refs = Vec::with_capacity(len);
        let mut current = Some(pimpl);
        while let Some(p) = current {
            refs.push(Arc::downgrade(p));
            current = p.chained_catalog.as_ref();
        }

        Self(refs.into())
    }

    pub fn upgrade(&self) -> Catalog {
        // Use the first catalog from the chain that is still alive
        for p in self.0.as_ref() {
            if let Some(pimpl) = Weak::upgrade(p) {
                return Catalog::new(pimpl);
            }
        }

        panic!("Catalog was already dropped")
    }

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
        let cat = self.upgrade();
        Spec::get(&cat, &ctx.push_resolve::<Spec>())
    }

    /// A short-hand for `get::<OneOf<T>>()`.
    #[inline(always)]
    pub fn get_one<Iface>(&self) -> Result<Arc<Iface>, InjectionError>
    where
        Iface: 'static + ?Sized + Send + Sync,
    {
        self.get::<OneOf<Iface>>()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl std::fmt::Debug for CatalogWeakRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for p in self.0.as_ref() {
            if let Some(p) = p.upgrade() {
                return write!(
                    f,
                    "CatalogWeakRef(0x{:x})",
                    p.as_ref() as *const CatalogImpl as usize
                );
            }
        }
        write!(f, "CatalogWeakRef(<dropped>)")
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
