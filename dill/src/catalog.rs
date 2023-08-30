use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use multimap::MultiMap;

use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct IfaceTypeId(pub TypeId);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct ImplTypeId(pub TypeId);

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct Catalog(pub(crate) Arc<CatalogInner>);

#[derive(Clone)]
pub(crate) struct CatalogInner {
    pub(crate) builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    pub(crate) bindings: MultiMap<IfaceTypeId, Binding>,
    pub(crate) chained_catalog: Option<Catalog>,
}

impl Catalog {
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

    pub fn builders<'a>(&'a self) -> Box<dyn Iterator<Item = &dyn Builder> + 'a> {
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
        let bindings = self.0.bindings.get_vec(&&iface_type);
        let it_bindings = TypecastBuilderIterator::new(bindings);

        if let Some(chained_catalog) = &self.0.chained_catalog {
            Box::new(it_bindings.chain(chained_catalog.builders_for::<Iface>()))
        } else {
            Box::new(it_bindings)
        }
    }

    pub fn get<Spec>(&self) -> Result<Spec::ReturnType, InjectionError>
    where
        Spec: DependencySpec + 'static,
    {
        Spec::get(self)
    }

    /// A short-hand for `get::<OneOf<T>>()`.
    pub fn get_one<Iface>(&self) -> Result<Arc<Iface>, InjectionError>
    where
        Iface: 'static + ?Sized + Send + Sync,
    {
        OneOf::<Iface>::get(self)
    }
}
