use std::any::TypeId;
use std::collections::HashMap;
use std::sync::Arc;

use multimap::MultiMap;

use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct IfaceTypeId(pub TypeId);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct ImplTypeId(pub TypeId);

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct CatalogImpl {
    pub(crate) builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    pub(crate) bindings: MultiMap<IfaceTypeId, Binding>,
    pub(crate) chained_catalog: Option<Arc<CatalogImpl>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl CatalogImpl {
    pub fn new(
        builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
        bindings: MultiMap<IfaceTypeId, Binding>,
        chained_catalog: Option<Arc<CatalogImpl>>,
    ) -> Self {
        Self {
            builders,
            bindings,
            chained_catalog,
        }
    }

    pub fn builders<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Builder> + 'a> {
        let it_builders = self.builders.values().map(|b| b.as_ref());
        if let Some(chained_catalog) = &self.chained_catalog {
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
        let bindings = self.bindings.get_vec(&iface_type);
        let it_bindings = TypecastBuilderIterator::new(bindings);

        if let Some(chained_catalog) = &self.chained_catalog {
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
        let bindings = self.bindings.get_vec(&iface_type);

        let it_bindings =
            TypecastPredicateBuilderIterator::new(bindings, move |b| b.metadata_contains(pred));

        if let Some(chained_catalog) = &self.chained_catalog {
            Box::new(it_bindings.chain(chained_catalog.builders_for_with_meta::<Iface, Meta>(pred)))
        } else {
            Box::new(it_bindings)
        }
    }
}
