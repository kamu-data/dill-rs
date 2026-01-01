use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

use crate::injection_context::InjectionContext;
use crate::*;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Builders are responsible for resolving dependencies and creating new
/// instances of a certain type. Builders typically create new instances for
/// every call, delegating the lifetime management to [Scope]s,
pub trait Builder: Send + Sync {
    /// [`TypeInfo`] of the type that this builder supplies
    fn instance_type(&self) -> TypeInfo;

    /// [`TypeInfo`] of the scope that caches the instances
    fn scope_type(&self) -> TypeInfo;

    /// Lists interfaces that the supplied type supports. Avoid using this
    /// low-level method directly - use [`BuilderExt`] convenience methods
    /// instead.
    fn interfaces(&self, clb: &mut dyn FnMut(&TypeInfo) -> bool);

    /// Lists dependencies required to construct an instance. Avoid using this
    /// low-level method directly - use [`BuilderExt`] convenience methods
    /// instead.
    fn dependencies(&self, clb: &mut dyn FnMut(&DependencyInfo) -> bool);

    /// Provider interface for accessing associated metadata. Avoid using this
    /// low-level method directly - use [`BuilderExt`] convenience methods
    /// instead.
    fn metadata<'a>(&'a self, clb: &mut dyn FnMut(&'a dyn std::any::Any) -> bool);

    /// Get an instance of the supplied type
    fn get_any(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>;

    /// Validate the dependency tree
    fn check(&self, cat: &Catalog, ctx: &InjectionContext) -> Result<(), ValidationError>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait BuilderExt {
    fn interfaces_get_all(&self) -> Vec<TypeInfo>;
    fn interfaces_contain<Iface: 'static>(&self) -> bool;
    fn interfaces_contain_type_id(&self, type_id: &TypeId) -> bool;

    fn dependencies_get_all(&self) -> Vec<DependencyInfo>;

    fn metadata_get_first<Meta: 'static>(&self) -> Option<&Meta>;
    fn metadata_find_first<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Option<&Meta>;
    fn metadata_get_all<Meta: 'static>(&self) -> Vec<&Meta>;
    fn metadata_find_all<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Vec<&Meta>;
    fn metadata_contains<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> bool;
}

impl<T: Builder + ?Sized> BuilderExt for T {
    fn interfaces_get_all(&self) -> Vec<TypeInfo> {
        let mut ret = Vec::new();
        self.interfaces(&mut |i| {
            ret.push(*i);
            true
        });
        ret
    }

    fn interfaces_contain<Iface: 'static>(&self) -> bool {
        let type_id = TypeId::of::<Iface>();
        self.interfaces_contain_type_id(&type_id)
    }

    fn interfaces_contain_type_id(&self, type_id: &TypeId) -> bool {
        let mut ret = false;
        self.interfaces(&mut |i| {
            if i.type_id == *type_id {
                ret = true;
                return false;
            }
            true
        });
        ret
    }

    fn dependencies_get_all(&self) -> Vec<DependencyInfo> {
        let mut ret = Vec::new();
        self.dependencies(&mut |i| {
            ret.push(*i);
            true
        });
        ret
    }

    fn metadata_get_first<Meta: 'static>(&self) -> Option<&Meta> {
        let mut ret: Option<&Meta> = None;
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>() {
                ret = Some(v);
                return false;
            }
            true
        });
        ret
    }

    fn metadata_find_first<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Option<&Meta> {
        let mut ret: Option<&Meta> = None;
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>()
                && pred(v)
            {
                ret = Some(v);
                return false;
            }
            true
        });
        ret
    }

    fn metadata_get_all<Meta: 'static>(&self) -> Vec<&Meta> {
        let mut ret: Vec<&Meta> = Vec::new();
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>() {
                ret.push(v);
            }
            true
        });
        ret
    }

    fn metadata_find_all<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Vec<&Meta> {
        let mut ret: Vec<&Meta> = Vec::new();
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>()
                && pred(v)
            {
                ret.push(v);
            }
            true
        });
        ret
    }

    fn metadata_contains<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> bool {
        let mut ret = false;
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>()
                && pred(v)
            {
                ret = true;
                return false;
            }
            true
        });
        ret
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait TypedBuilder<T: Send + Sync + ?Sized>: Builder {
    /// Called to get an instance of the component, respecting the lifetime
    /// defined by the scope
    fn get(&self, cat: &Catalog) -> Result<Arc<T>, InjectionError> {
        self.get_with_context(cat, &InjectionContext::new_root())
    }

    /// Called to get an instance of the component, respecting the lifetime
    /// defined by the scope
    fn get_with_context(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<T>, InjectionError>;

    /// Called during registration to automatically bind this builder to all
    /// interfaces this component implements
    fn bind_interfaces(&self, cat: &mut CatalogBuilder);
}

pub trait TypedBuilderExt<T: Send + Sync + ?Sized>: TypedBuilder<T> {
    /// Stops builder from auto-registering the default interfaces, allowing a
    /// fine-grain control over the binding
    fn without_default_interfaces(self) -> impl TypedBuilder<T>;
}

pub trait TypedBuilderCast<I: Send + Sync + ?Sized> {
    fn cast(self) -> impl TypedBuilder<I>;
}

/// Allows [CatalogBuilder::add()] to accept types with associated builder
pub trait Component {
    type Impl: Send + Sync;
    type Builder: TypedBuilder<Self::Impl>;

    fn builder() -> Self::Builder;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TypeInfo {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl TypeInfo {
    pub fn of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: std::any::TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DependencyInfo {
    pub type_info: TypeInfo,
    pub spec: TypeInfo,
}

impl DependencyInfo {
    pub fn of<T: ?Sized + 'static, Spec: DependencySpec + 'static>() -> Self {
        Self {
            type_info: TypeInfo::of::<T>(),
            spec: TypeInfo::of::<Spec>(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

impl<Bld, Impl> TypedBuilderExt<Impl> for Bld
where
    Impl: Send + Sync,
    Bld: TypedBuilder<Impl>,
{
    fn without_default_interfaces(self) -> impl TypedBuilder<Impl> {
        TypedBuilderWithoutDefaultInterfaces(self)
    }
}

/// A wrapper builder that stops it from auto-registering default interfaces
pub struct TypedBuilderWithoutDefaultInterfaces<Bld>(Bld);

impl<Bld> Builder for TypedBuilderWithoutDefaultInterfaces<Bld>
where
    Bld: Builder,
{
    fn instance_type(&self) -> TypeInfo {
        self.0.instance_type()
    }

    fn scope_type(&self) -> TypeInfo {
        self.0.scope_type()
    }

    fn interfaces(&self, clb: &mut dyn FnMut(&TypeInfo) -> bool) {
        self.0.interfaces(clb);
    }

    fn dependencies(&self, clb: &mut dyn FnMut(&DependencyInfo) -> bool) {
        self.0.dependencies(clb);
    }

    fn metadata<'a>(&'a self, clb: &mut dyn FnMut(&'a dyn std::any::Any) -> bool) {
        self.0.metadata(clb);
    }

    fn get_any(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        self.0.get_any(cat, ctx)
    }

    fn check(&self, cat: &Catalog, ctx: &InjectionContext) -> Result<(), ValidationError> {
        self.0.check(cat, ctx)
    }
}

impl<Bld, Impl> TypedBuilder<Impl> for TypedBuilderWithoutDefaultInterfaces<Bld>
where
    Impl: Send + Sync,
    Bld: TypedBuilder<Impl>,
{
    fn get_with_context(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<Impl>, InjectionError> {
        self.0.get_with_context(cat, ctx)
    }

    fn bind_interfaces(&self, _cat: &mut CatalogBuilder) {}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Arc<T> can infinitely produce clones of itself and therefore is a builder
impl<Impl> Builder for Arc<Impl>
where
    Impl: Send + Sync + 'static,
{
    fn instance_type(&self) -> TypeInfo {
        TypeInfo::of::<Impl>()
    }

    fn scope_type(&self) -> TypeInfo {
        TypeInfo::of::<crate::scopes::Singleton>()
    }

    fn interfaces(&self, _clb: &mut dyn FnMut(&TypeInfo) -> bool) {}

    fn dependencies(&self, _clb: &mut dyn FnMut(&DependencyInfo) -> bool) {}

    fn metadata<'a>(&'a self, _clb: &mut dyn FnMut(&'a dyn Any) -> bool) {}

    fn get_any(
        &self,
        _cat: &Catalog,
        _ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(self.clone())
    }

    fn check(&self, _cat: &Catalog, _ctx: &InjectionContext) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Impl> TypedBuilder<Impl> for Arc<Impl>
where
    Impl: Send + Sync + 'static,
{
    fn get_with_context(
        &self,
        _cat: &Catalog,
        _ctx: &InjectionContext,
    ) -> Result<Arc<Impl>, InjectionError> {
        Ok(self.clone())
    }

    fn bind_interfaces(&self, _cat: &mut CatalogBuilder) {}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Fn() -> Arc<T> acts as a builder
impl<Fct, Impl> Builder for Fct
where
    Fct: Fn() -> Arc<Impl> + Send + Sync,
    Impl: Send + Sync + 'static,
{
    fn instance_type(&self) -> TypeInfo {
        TypeInfo::of::<Impl>()
    }

    fn scope_type(&self) -> TypeInfo {
        TypeInfo::of::<crate::scopes::Transient>()
    }

    fn interfaces(&self, _clb: &mut dyn FnMut(&TypeInfo) -> bool) {}

    fn dependencies(&self, _clb: &mut dyn FnMut(&DependencyInfo) -> bool) {}

    fn metadata<'a>(&'a self, _clb: &mut dyn FnMut(&'a dyn Any) -> bool) {}

    fn get_any(
        &self,
        _cat: &Catalog,
        _ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(self())
    }

    fn check(&self, _cat: &Catalog, _ctx: &InjectionContext) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Fct, Impl> TypedBuilder<Impl> for Fct
where
    Fct: Fn() -> Arc<Impl> + Send + Sync,
    Impl: Send + Sync + 'static,
{
    fn get_with_context(
        &self,
        _cat: &Catalog,
        _ctx: &InjectionContext,
    ) -> Result<Arc<Impl>, InjectionError> {
        Ok(self())
    }

    fn bind_interfaces(&self, _cat: &mut CatalogBuilder) {}
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct LazyBuilder<Fct, Impl>
where
    Fct: FnOnce() -> Impl,
    Impl: 'static + Send + Sync,
{
    state: Mutex<LazyBuilderState<Fct, Impl>>,
}

struct LazyBuilderState<Fct, Impl> {
    factory: Option<Fct>,
    instance: Option<Arc<Impl>>,
}

impl<Fct, Impl> LazyBuilder<Fct, Impl>
where
    Fct: FnOnce() -> Impl,
    Impl: 'static + Send + Sync,
{
    pub fn new(factory: Fct) -> Self {
        Self {
            state: Mutex::new(LazyBuilderState {
                factory: Some(factory),
                instance: None,
            }),
        }
    }
}

impl<Fct, Impl> Builder for LazyBuilder<Fct, Impl>
where
    Fct: FnOnce() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn instance_type(&self) -> TypeInfo {
        TypeInfo::of::<Impl>()
    }

    fn scope_type(&self) -> TypeInfo {
        TypeInfo::of::<crate::scopes::Transient>()
    }

    fn interfaces(&self, _clb: &mut dyn FnMut(&TypeInfo) -> bool) {}

    fn dependencies(&self, _clb: &mut dyn FnMut(&DependencyInfo) -> bool) {}

    fn metadata<'a>(&'a self, _clb: &mut dyn FnMut(&'a dyn Any) -> bool) {}

    fn get_any(
        &self,
        cat: &Catalog,
        ctx: &InjectionContext,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(TypedBuilder::get_with_context(self, cat, ctx)?)
    }

    fn check(&self, _cat: &Catalog, _ctx: &InjectionContext) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Fct, Impl> TypedBuilder<Impl> for LazyBuilder<Fct, Impl>
where
    Fct: FnOnce() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn get_with_context(
        &self,
        _cat: &Catalog,
        _ctx: &InjectionContext,
    ) -> Result<Arc<Impl>, InjectionError> {
        let mut s = self.state.lock().unwrap();
        if let Some(inst) = s.instance.as_ref() {
            Ok(inst.clone())
        } else {
            let factory = s.factory.take().unwrap();
            let inst = Arc::new(factory());
            s.instance = Some(inst.clone());
            Ok(inst)
        }
    }

    fn bind_interfaces(&self, _cat: &mut CatalogBuilder) {}
}
