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
        if builders.next().is_some() {
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
        // TODO: Avoid wrapping in Arc?
        Ok(Arc::new(cat.clone()))
    }

    fn check(_: &Catalog) -> Result<(), InjectionError> {
        Ok(())
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

/////////////////////////////////////////////////////////////////////////////////////////
// Maybe
/////////////////////////////////////////////////////////////////////////////////////////

/// Returns `None` if an optional dependency is not registered
pub struct Maybe<Inner: DependencySpec> {
    _dummy: PhantomData<Inner>,
}

impl<Inner: DependencySpec> DependencySpec for Maybe<Inner> {
    type ReturnType = Option<Inner::ReturnType>;

    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        match Inner::get(cat) {
            Ok(v) => Ok(Some(v)),
            Err(InjectionError::Unregistered(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }

    fn check(cat: &Catalog) -> Result<(), InjectionError> {
        match Inner::check(cat) {
            Ok(()) => Ok(()),
            Err(InjectionError::Unregistered(_)) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// Lazy
/////////////////////////////////////////////////////////////////////////////////////////

/// Delays the instantiation of a component untill explicitly requested.
///
/// See [`crate::lazy::Lazy`] documentation for details.
pub struct Lazy<Inner: DependencySpec> {
    _dummy: PhantomData<Inner>,
}

impl<Inner: DependencySpec> DependencySpec for Lazy<Inner> {
    type ReturnType = crate::lazy::Lazy<Inner::ReturnType>;

    #[cfg(not(feature = "tokio"))]
    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        let cat = cat.clone();
        Ok(crate::lazy::Lazy::new(move || Inner::get(&cat)))
    }

    #[cfg(feature = "tokio")]
    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        // Lazy<T> will store the clone of a catalog it was initially created with
        // It will however first attempt to resolve a current catalog if scope feature
        // is used and only use the former as a fallback.
        let fallback_cat = cat.clone();
        Ok(crate::lazy::Lazy::new(move || match crate::CURRENT_CATALOG
            .try_with(|cat| Inner::get(cat))
        {
            Ok(v) => v,
            Err(_) => Inner::get(&fallback_cat),
        }))
    }

    fn check(cat: &Catalog) -> Result<(), InjectionError> {
        Inner::check(cat)
    }
}
