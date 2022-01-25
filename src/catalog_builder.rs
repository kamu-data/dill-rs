use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    marker::Unsize,
    sync::Arc,
};

use multimap::MultiMap;

use super::catalog::*;
use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CatalogBuilder {
    builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    bindings: MultiMap<IfaceTypeId, Binding>,
}

impl CatalogBuilder {
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
            bindings: MultiMap::new(),
        }
    }

    pub fn add<Bld: BuilderLike>(&mut self) -> &mut Self {
        Bld::register(self);
        self
    }

    pub fn add_builder<Bld, Impl>(&mut self, builder: Bld) -> &mut Self
    where
        Impl: 'static + Send + Sync,
        Bld: TypedBuilder<Impl> + 'static,
    {
        let key = ImplTypeId(TypeId::of::<Impl>());
        if self.builders.contains_key(&key) {
            panic!(
                "Builder for type {} is already registered",
                type_name::<Impl>()
            );
        }

        let builder = Arc::new(builder);
        self.builders.insert(key, builder.clone());

        self.bindings.insert(
            IfaceTypeId(TypeId::of::<Impl>()),
            Binding::new(
                Arc::new(TypeCaster::<Impl> {
                    // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl` instances
                    cast_arc: |v| v.downcast().unwrap(),
                }),
                builder,
            ),
        );

        self
    }

    // TODO: Replace with generic add<B: Into<Builder>>?
    pub fn add_factory<Fct, Impl>(&mut self, factory: Fct) -> &mut Self
    where
        Fct: 'static + Fn() -> Impl + Send + Sync,
        Impl: 'static + Send + Sync,
    {
        self.add_builder(Factory::new(factory));
        self
    }

    // TODO: Replace with generic add<B: Into<Builder>>?
    pub fn add_value<'a, Impl>(&'a mut self, value: Impl) -> &mut Self
    where
        Impl: 'static + Send + Sync,
    {
        self.add_builder(Prebuilt::from_value(value));
        self
    }

    // TODO: WTF is Unsize
    pub fn bind<Iface, Impl>(&mut self) -> &mut Self
    where
        Iface: 'static + ?Sized,
        Impl: 'static + Send + Sync + Unsize<Iface>,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let impl_type = ImplTypeId(TypeId::of::<Impl>());

        let builder = self.builders.get(&impl_type);
        if builder.is_none() {
            panic!("Interface type {} is not registered", type_name::<Iface>());
        }

        self.bindings.insert(
            iface_type,
            Binding::new(
                Arc::new(TypeCaster::<Iface> {
                    cast_arc: |v| {
                        // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl` instances
                        let s: Arc<Impl> = v.downcast().unwrap();
                        let t: Arc<Iface> = s;
                        t
                    },
                }),
                builder.unwrap().clone(),
            ),
        );

        self
    }

    pub fn build(&mut self) -> Catalog {
        let mut builders = HashMap::new();
        let mut bindings = MultiMap::new();
        std::mem::swap(&mut self.builders, &mut builders);
        std::mem::swap(&mut self.bindings, &mut bindings);
        Catalog::new(builders, bindings)
    }
}
