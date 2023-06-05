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
}

impl Catalog {
    pub(crate) fn new(
        builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
        bindings: MultiMap<IfaceTypeId, Binding>,
    ) -> Self {
        Self(Arc::new(CatalogInner { builders, bindings }))
    }

    pub fn builders(&self) -> impl Iterator<Item = &dyn Builder> {
        self.0.builders.values().map(|b| b.as_ref())
    }

    pub fn builders_for<'a, Iface>(&'a self) -> impl Iterator<Item = TypecastBuilder<'a, Iface>>
    where
        Iface: 'static + ?Sized,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let bindings = self.0.bindings.get_vec(&&iface_type);
        TypecastBuilderIterator::new(bindings)
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
