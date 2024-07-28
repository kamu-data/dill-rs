use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

/// Builders are responsible for resolving dependencies and creating new
/// instances of a certain type. Builders typically create new instances for
/// every call, delegating the lifetime management to [Scope]s,
pub trait Builder: Send + Sync {
    /// [`TypeId`] of the type that this builder supplies
    fn instance_type_id(&self) -> TypeId;

    /// Name of the type that this builder supplies in the `mod1::mod2::Typ`
    /// format
    fn instance_type_name(&self) -> &'static str;

    // TODO: Avoid allocating
    /// Lists interfaces that the supplied type supports
    fn interfaces(&self) -> Vec<InterfaceDesc>;

    /// Provider interface for accessing associated metadata. Avoid using this
    /// low-level method directly - use [`BuilderExt`] convenience methods
    /// instead.
    #[allow(unused_variables)]
    fn metadata<'a>(&'a self, clb: &mut dyn FnMut(&'a dyn std::any::Any) -> bool) {}

    /// Get an instance of the supplied type
    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>;

    /// Validate the dependency tree
    fn check(&self, cat: &Catalog) -> Result<(), ValidationError>;
}

/////////////////////////////////////////////////////////////////////////////////////////

pub trait BuilderExt {
    fn metadata_get_first<Meta: 'static>(&self) -> Option<&Meta>;
    fn metadata_find_first<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Option<&Meta>;
    fn metadata_get_all<Meta: 'static>(&self) -> Vec<&Meta>;
    fn metadata_find_all<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> Vec<&Meta>;
    fn metadata_contains<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> bool;
}

impl<T: Builder + ?Sized> BuilderExt for T {
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
            if let Some(v) = m.downcast_ref::<Meta>() {
                if pred(v) {
                    ret = Some(v);
                    return false;
                }
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
            if let Some(v) = m.downcast_ref::<Meta>() {
                if pred(v) {
                    ret.push(v);
                }
            }
            true
        });
        ret
    }

    fn metadata_contains<Meta: 'static>(&self, pred: impl Fn(&Meta) -> bool) -> bool {
        let mut ret = false;
        self.metadata(&mut |m| {
            if let Some(v) = m.downcast_ref::<Meta>() {
                if pred(v) {
                    ret = true;
                    return false;
                }
            }
            true
        });
        ret
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub trait TypedBuilder<T: Send + Sync>: Builder {
    fn get(&self, cat: &Catalog) -> Result<Arc<T>, InjectionError>;
}

/// Allows [CatalogBuilder::add()] to accept types with associated builder
pub trait Component {
    type Builder: Builder;
    fn builder() -> Self::Builder;
    fn register(cat: &mut CatalogBuilder);
}

#[derive(Debug, Copy, Clone)]
pub struct InterfaceDesc {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Arc<T> can infinitely produce clones of itself and therefore is a builder
impl<Impl> Builder for Arc<Impl>
where
    Impl: Send + Sync + 'static,
{
    fn instance_type_id(&self) -> TypeId {
        TypeId::of::<Impl>()
    }

    fn instance_type_name(&self) -> &'static str {
        std::any::type_name::<Impl>()
    }

    fn interfaces(&self) -> Vec<InterfaceDesc> {
        Vec::new()
    }

    fn get(&self, _cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(self.clone())
    }

    fn check(&self, _cat: &Catalog) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Impl> TypedBuilder<Impl> for Arc<Impl>
where
    Impl: Send + Sync + 'static,
{
    fn get(&self, _cat: &Catalog) -> Result<Arc<Impl>, InjectionError> {
        Ok(self.clone())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Fn() -> Arc<T> acts as a builder
impl<Fct, Impl> Builder for Fct
where
    Fct: Fn() -> Arc<Impl> + Send + Sync,
    Impl: Send + Sync + 'static,
{
    fn instance_type_id(&self) -> TypeId {
        TypeId::of::<Impl>()
    }

    fn instance_type_name(&self) -> &'static str {
        std::any::type_name::<Impl>()
    }

    fn interfaces(&self) -> Vec<InterfaceDesc> {
        Vec::new()
    }

    fn get(&self, _cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(self())
    }

    fn check(&self, _cat: &Catalog) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Fct, Impl> TypedBuilder<Impl> for Fct
where
    Fct: Fn() -> Arc<Impl> + Send + Sync,
    Impl: Send + Sync + 'static,
{
    fn get(&self, _cat: &Catalog) -> Result<Arc<Impl>, InjectionError> {
        Ok(self())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

pub(crate) struct Lazy<Fct, Impl>
where
    Fct: FnOnce() -> Impl,
    Impl: 'static + Send + Sync,
{
    state: Mutex<LazyState<Fct, Impl>>,
}

struct LazyState<Fct, Impl> {
    factory: Option<Fct>,
    instance: Option<Arc<Impl>>,
}

impl<Fct, Impl> Lazy<Fct, Impl>
where
    Fct: FnOnce() -> Impl,
    Impl: 'static + Send + Sync,
{
    pub fn new(factory: Fct) -> Self {
        Self {
            state: Mutex::new(LazyState {
                factory: Some(factory),
                instance: None,
            }),
        }
    }
}

impl<Fct, Impl> Builder for Lazy<Fct, Impl>
where
    Fct: FnOnce() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn instance_type_id(&self) -> TypeId {
        TypeId::of::<Impl>()
    }

    fn instance_type_name(&self) -> &'static str {
        std::any::type_name::<Impl>()
    }

    fn interfaces(&self) -> Vec<InterfaceDesc> {
        Vec::new()
    }

    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(TypedBuilder::get(self, cat)?)
    }

    fn check(&self, _cat: &Catalog) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl<Fct, Impl> TypedBuilder<Impl> for Lazy<Fct, Impl>
where
    Fct: FnOnce() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn get(&self, _cat: &Catalog) -> Result<Arc<Impl>, InjectionError> {
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
}
