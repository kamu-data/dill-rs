use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::marker::Unsize;
use std::sync::Arc;

use multimap::MultiMap;

use super::catalog::*;
use crate::injection_context::InjectionContext;
use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CatalogBuilder {
    builders: HashMap<ImplTypeId, Arc<dyn Builder>>,
    bindings: MultiMap<IfaceTypeId, Binding>,
    chained_catalog: Option<Arc<CatalogImpl>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
            chained_catalog: Some(chained_catalog.0.clone()),
        }
    }

    /// Registers a component using its associated builder.
    ///
    /// Note that unlike [CatalogBuilder::add_builder()] this will also bind the
    /// implementation to component's default interfaces.
    pub fn add<C>(&mut self) -> &mut Self
    where
        C: 'static + Component,
    {
        let builder = C::builder();
        self.add_builder(builder);
        self
    }

    pub fn add_builder<Bld, Impl>(&mut self, builder: Bld) -> &mut Self
    where
        Impl: 'static + Send + Sync,
        Bld: 'static + TypedBuilder<Impl>,
    {
        let key = ImplTypeId(TypeId::of::<Impl>());
        if self.builders.contains_key(&key) {
            panic!(
                "Builder for type {} is already registered",
                type_name::<Impl>()
            );
        }

        let builder_arc = Arc::new(builder);
        self.builders.insert(key, builder_arc.clone());

        // Bind implementation
        self.bindings.insert(
            IfaceTypeId(TypeId::of::<Impl>()),
            Binding::new(
                Arc::new(TypeCaster::<Impl> {
                    // SAFETY: `TypeCaster<Iface>` is guaranteed to be invoked only on the `Impl`
                    // instances
                    cast_arc: |v| v.downcast().unwrap(),
                }),
                builder_arc.clone(),
            ),
        );

        // To call the correct `TypedBuilder<Impl>::bind_interfaces()` method,
        // we need to call it exactly on `TypedBuilder<Impl>` type,
        // not `Arc<TypedBuilder<Impl>>`, which has an empty (default) implementation
        (*builder_arc).bind_interfaces(self);

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
        Catalog::new(Arc::new(CatalogImpl::new(
            builders,
            bindings,
            self.chained_catalog.take(),
        )))
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
        const SCOPE_COMPAT: [TypeId; 4] = [
            TypeId::of::<Agnostic>(),
            TypeId::of::<Transient>(),
            TypeId::of::<Transaction>(),
            TypeId::of::<Singleton>(),
        ];

        let get_binding = |t: &IfaceTypeId| {
            if let Some(v) = self.bindings.get(t) {
                return Some(v);
            }

            let mut chained = self.chained_catalog.as_ref();
            while let Some(c) = chained {
                if let Some(v) = c.bindings.get(t) {
                    return Some(v);
                }
                chained = c.chained_catalog.as_ref();
            }
            None
        };

        let mut errors = Vec::new();
        let mut validated = std::collections::HashSet::<TypeId>::new();

        for b in self.builders.values() {
            let inst = b.instance_type();
            let inst_scope = b.scope_type();

            if validated.contains(&inst.id) {
                continue;
            }

            for dep in b.dependencies_get_all() {
                if dep.is_bound {
                    // OK: provided explicitly
                } else if let Some(bind) = get_binding(&IfaceTypeId(dep.iface.id)) {
                    let dep_scope = bind.builder.scope_type();

                    if dep_scope.id == TypeId::of::<Agnostic>() {
                        // OK: Agnostic is safe to inject in any scope
                        continue;
                    }

                    // TODO: Make scope compatibility checks more robust
                    let i = SCOPE_COMPAT
                        .iter()
                        .position(|t| *t == inst_scope.id)
                        .unwrap();
                    let d = SCOPE_COMPAT
                        .iter()
                        .position(|t| *t == dep_scope.id)
                        .unwrap();

                    if i > d {
                        let err = InjectionError::ScopeInversion(Box::new(ScopeInversionError {
                            inst_type: inst,
                            inst_scope,
                            inst_dep: dep,
                            dep_type: bind.builder.instance_type(),
                            dep_scope,
                            injection_stack: InjectionContext::new_root()
                                .push_build(b.as_ref())
                                .push(InjectionStackFrame::Resolve {
                                    spec: dep.spec,
                                    iface: dep.iface,
                                })
                                .to_stack(),
                        }));
                        errors.push(err);
                    }
                } else if dep.iface.id == TypeId::of::<Catalog>()
                    || dep.iface.id == TypeId::of::<CatalogWeakRef>()
                {
                    // OK: self-injection of a catalog
                } else {
                    // TODO: Make spec identification more robust
                    let spec = dep
                        .spec
                        .name
                        .replace(dep.iface.name, "")
                        .replace("dill::specs::", "");
                    match spec.as_str() {
                        "Maybe<OneOf<>>" | "AllOf<>" => {
                            // OK: dependency is optional
                        }
                        _ => {
                            let err = InjectionError::Unregistered(UnregisteredTypeError {
                                dep_type: dep.iface,
                                injection_stack: InjectionContext::new_root()
                                    .push_build(b.as_ref())
                                    .push(InjectionStackFrame::Resolve {
                                        spec: dep.spec,
                                        iface: dep.iface,
                                    })
                                    .to_stack(),
                            });
                            errors.push(err);
                        }
                    }
                }
            }

            validated.insert(inst.id);
        }

        // Sort and deduplicate by type
        errors.sort_by_key(|e| match e {
            InjectionError::Unregistered(err) => err.dep_type.id,
            InjectionError::Ambiguous(err) => err.dep_type.id,
            InjectionError::ScopeInversion(err) => err.dep_type.id,
        });
        errors.dedup_by_key(|e| match e {
            InjectionError::Unregistered(err) => err.dep_type.id,
            InjectionError::Ambiguous(err) => err.dep_type.id,
            InjectionError::ScopeInversion(err) => err.dep_type.id,
        });

        if !errors.is_empty() {
            Err(ValidationError { errors })
        } else {
            Ok(())
        }
    }
}
