use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::Unsize;
use std::sync::Arc;

use multimap::MultiMap;

use super::catalog::*;
use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CatalogBuilder {
    builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    bindings: MultiMap<IfaceTypeId, Binding>,
    chained_catalog: Option<Catalog>,
}

/////////////////////////////////////////////////////////////////////////////////////////

impl Default for CatalogBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogBuilder {
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
            bindings: MultiMap::new(),
            chained_catalog: None,
        }
    }

    pub fn new_chained(chained_catalog: &Catalog) -> Self {
        Self {
            builders: HashMap::new(),
            bindings: MultiMap::new(),
            chained_catalog: Some(chained_catalog.clone()),
        }
    }

    /// Registers a component using its associated builder.
    ///
    /// Note that unlike [CatalogBuilder::add_builder()] this will also bind the
    /// implementation to component's default interfaces.
    pub fn add<C: Component>(&mut self) -> &mut Self {
        C::register(self);
        self
    }

    pub fn add_builder<Bld, Impl>(&mut self, builder: Bld) -> &mut Self
    where
        Impl: 'static + Send + Sync,
        Bld: TypedBuilder<Impl> + TypedBuilderInterfaceBinder + 'static,
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
                    // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl`
                    // instances
                    cast_arc: |v| v.downcast().unwrap(),
                }),
                builder,
            ),
        );

        Bld::bind_interfaces(self);

        self
    }

    pub fn add_value<Impl>(&mut self, value: Impl) -> &mut Self
    where
        Impl: 'static + Send + Sync,
    {
        self.add_builder(Arc::new(value));
        self
    }

    /// Uses the provided factory once and caches the instance in a [Singleton]
    /// scope
    pub fn add_value_lazy<Fct, Impl>(&mut self, factory: Fct) -> &mut Self
    where
        Fct: FnOnce() -> Impl + Send + Sync + 'static,
        Impl: Send + Sync + 'static,
    {
        self.add_builder(LazyBuilder::new(factory));
        self
    }

    pub fn bind<Iface, Impl>(&mut self) -> &mut Self
    where
        Iface: 'static + ?Sized,
        Impl: 'static + Send + Sync + Unsize<Iface>,
    {
        let iface_type = IfaceTypeId(TypeId::of::<Iface>());
        let impl_type = ImplTypeId(TypeId::of::<Impl>());

        let builder = self.builders.get(&impl_type);
        if builder.is_none() {
            panic!("Builder for type {} is not registered", type_name::<Impl>());
        }

        self.bindings.insert(
            iface_type,
            Binding::new(
                Arc::new(TypeCaster::<Iface> {
                    cast_arc: |v| {
                        // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the
                        // `Impl` instances
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
        Catalog::new(builders, bindings, self.chained_catalog.take())
    }

    /// Validates the dependency graph returning a combined error.
    ///
    /// In case some of your types are registered dynamically you can
    /// [ValidationErrorExt::ignore()] method which is implemented on the
    /// Result type (you need to import the trait).
    ///
    /// Example:
    /// ```
    /// use dill::*;
    /// trait MyDynamicType {}
    ///
    /// let mut b = CatalogBuilder::new();
    /// // Populate the builder
    /// b.validate()
    ///  .ignore::<dyn MyDynamicType>()
    ///  .unwrap();
    /// ```
    pub fn validate(&mut self) -> Result<(), ValidationError> {
        // TODO: Should return a validation report type that will track
        // - Unresolved dependencies
        // - Ambiguous dependencies
        // - Missing dependencies with defaults
        // - AllOf that don't resolve to anything
        //
        // Users will then be able to specify whether to treat them as errors / warnings
        // or have them ignored.

        let mut errors = Vec::new();

        // TODO: Avoid allocations when constructing a temporary catalog
        let cat = self.build();
        for builder in cat.builders() {
            if let Err(mut err) = builder.check(&cat) {
                errors.append(&mut err.errors);
            }
        }

        // Sort and deduplicate by type
        errors.sort_by_key(|e| match e {
            InjectionError::Unregistered(err) => err.type_id,
            InjectionError::Ambiguous(err) => err.type_id,
        });
        errors.dedup_by_key(|e| match e {
            InjectionError::Unregistered(err) => err.type_id,
            InjectionError::Ambiguous(err) => err.type_id,
        });

        // Return builder to its original state
        let mut cat = Arc::into_inner(cat.0).unwrap();
        std::mem::swap(&mut self.builders, &mut cat.builders);
        std::mem::swap(&mut self.bindings, &mut cat.bindings);
        self.chained_catalog = cat.chained_catalog.take();

        if !errors.is_empty() {
            Err(ValidationError { errors })
        } else {
            Ok(())
        }
    }
}
