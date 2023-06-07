use std::any::{Any, TypeId};
use std::sync::{Arc, Mutex};

use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

/// Builders are responsible for resolving dependencies and creating new
/// instances of a certain type. Builders typically create new instances for
/// every call, delegating the lifetime management to [Scope]s,
pub trait Builder: Send + Sync {
    fn instance_type_id(&self) -> TypeId;
    fn instance_type_name(&self) -> &'static str;
    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>;
    fn check(&self, cat: &Catalog) -> Result<(), ValidationError>;
}

pub trait TypedBuilder<T: Send + Sync>: Builder {
    fn get(&self, cat: &Catalog) -> Result<Arc<T>, InjectionError>;
}

/// Allows [CatalogBuilder::add()] to accept both impl types with associated
/// builder and custom builders
pub trait BuilderLike {
    type Builder: Builder;
    fn register(cat: &mut CatalogBuilder);
    fn builder() -> Self::Builder;
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Used to create an instance of a default builder for a component.
/// This instance can be then be parametrized before adding it into the
/// [CatalogBuilder].
///
/// # Examples
///
/// ```
/// use dill::*;
///
/// #[component]
/// struct ConnectionPool {
///     host: String,
///     port: u16,
/// }
///
/// let catalog = CatalogBuilder::new()
///     .add_builder(
///         builder_for::<ConnectionPool>()
///             .with_host("foo".to_owned())
///             .with_port(8080)
///     );
/// ```
pub fn builder_for<B: BuilderLike>() -> B::Builder {
    B::builder()
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
