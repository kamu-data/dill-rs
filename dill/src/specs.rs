use std::marker::PhantomData;
use std::sync::Arc;

use crate::{Catalog, InjectionError};

/////////////////////////////////////////////////////////////////////////////////////////
// DependencySpec
/////////////////////////////////////////////////////////////////////////////////////////

/// Specifies a particular way of resolving a dependency using the [`Catalog`]
pub trait DependencySpec {
    type ReturnType;
    // Resolve and create instances
    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError>;
    // Only resolve builders without instantiating and report errors
    fn check(cat: &Catalog) -> Result<(), InjectionError>;
}

/////////////////////////////////////////////////////////////////////////////////////////
// OneOf
/////////////////////////////////////////////////////////////////////////////////////////

/// Builds a single instance of type implementing specific interface. Will
/// return an error if no implementations or multiple implementations were
/// found.
pub struct OneOf<Iface>
where
    Iface: 'static + ?Sized + Send + Sync,
{
    _dummy: PhantomData<Iface>,
}

impl<Iface> DependencySpec for OneOf<Iface>
where
    Iface: 'static + ?Sized + Send + Sync,
{
    type ReturnType = Arc<Iface>;

    default fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        let mut builders = cat.builders_for::<Iface>();
        if let Some(first) = builders.next() {
            if builders.next().is_some() {
                Err(InjectionError::ambiguous::<Iface>())
            } else {
                first.get(cat)
            }
        } else {
            Err(InjectionError::unregistered::<Iface>())
        }
    }

    default fn check(cat: &Catalog) -> Result<(), InjectionError> {
        let mut builders = cat.builders_for::<Iface>();
        if let Some(_) = builders.next() {
            if builders.next().is_some() {
                Err(InjectionError::ambiguous::<Iface>())
            } else {
                Ok(())
            }
        } else {
            Err(InjectionError::unregistered::<Iface>())
        }
    }
}

impl DependencySpec for OneOf<Catalog> {
    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        Ok(Arc::new(cat.clone()))
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// AllOf
/////////////////////////////////////////////////////////////////////////////////////////

/// Builds all instances that implement a specific interface, returning a
/// [`Vec`].
pub struct AllOf<Iface>
where
    Iface: 'static + ?Sized,
{
    _dummy: PhantomData<Iface>,
}

impl<Iface> DependencySpec for AllOf<Iface>
where
    Iface: 'static + ?Sized,
{
    type ReturnType = Vec<Arc<Iface>>;

    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        cat.builders_for::<Iface>().map(|b| b.get(cat)).collect()
    }

    fn check(_cat: &Catalog) -> Result<(), InjectionError> {
        Ok(())
    }
}
