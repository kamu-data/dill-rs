use std::{
    any::{Any, TypeId},
    sync::Arc,
};

use crate::*;

/////////////////////////////////////////////////////////////////////////////////////////

/// Builders are responsible for resolving dependencies and creating new instances of a certain type.
/// Builders typically create new instances for every call, delegating the lifetime management to [`Scope`]s,
pub trait Builder: Send + Sync {
    fn instance_type_id(&self) -> TypeId;
    fn instance_type_name(&self) -> &'static str;
    fn get(&self, cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>;
}

pub trait TypedBuilder<T: Send + Sync>: Builder {
    fn get(&self, cat: &Catalog) -> Result<Arc<T>, InjectionError>;
}

/// Allows [`CatalogBuilder::add()`] to accept both impl types with associated builder and custom builders
pub trait BuilderLike {
    type Builder: Builder;
    fn register(cat: &mut CatalogBuilder);
    fn builder() -> Self::Builder;
}

/////////////////////////////////////////////////////////////////////////////////////////

/// Used to create an instance of a default builder for a component.
/// This instance can be then be parametrized before adding it into the [`CatalogBuilder`].
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

pub struct Prebuilt<T>
where
    T: 'static + Send + Sync,
{
    value: Arc<T>,
}

impl<T> Prebuilt<T>
where
    T: 'static + Send + Sync,
{
    pub fn from_value(value: T) -> Self {
        Self {
            value: Arc::new(value),
        }
    }

    pub fn from_shared(value: Arc<T>) -> Self {
        Self { value }
    }
}

impl<T> Builder for Prebuilt<T>
where
    T: 'static + Send + Sync,
{
    fn instance_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn instance_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn get(&self, _cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(self.value.clone())
    }
}

impl<T> TypedBuilder<T> for Prebuilt<T>
where
    T: 'static + Send + Sync,
{
    fn get(&self, _cat: &Catalog) -> Result<Arc<T>, InjectionError> {
        Ok(self.value.clone())
    }
}

/////////////////////////////////////////////////////////////////////////////////////////

// TODO: Implement Builder trait for any `Fn() -> T` without wrapping?
pub struct Factory<Fct, Impl>
where
    Fct: Fn() -> Impl,
    Impl: 'static + Send + Sync,
{
    factory: Fct,
}

impl<Fct, Impl> Factory<Fct, Impl>
where
    Fct: Fn() -> Impl,
    Impl: 'static + Send + Sync,
{
    pub fn new(factory: Fct) -> Self {
        Self { factory }
    }
}

impl<Fct, Impl> Builder for Factory<Fct, Impl>
where
    Fct: Fn() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn instance_type_id(&self) -> TypeId {
        TypeId::of::<Impl>()
    }

    fn instance_type_name(&self) -> &'static str {
        std::any::type_name::<Impl>()
    }

    fn get(&self, _cat: &Catalog) -> Result<Arc<dyn Any + Send + Sync>, InjectionError> {
        Ok(Arc::new((self.factory)()))
    }
}

impl<Fct, Impl> TypedBuilder<Impl> for Factory<Fct, Impl>
where
    Fct: Fn() -> Impl + Send + Sync,
    Impl: 'static + Send + Sync,
{
    fn get(&self, _cat: &Catalog) -> Result<Arc<Impl>, InjectionError> {
        Ok(Arc::new((self.factory)()))
    }
}
