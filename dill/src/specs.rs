use std::marker::PhantomData;
use std::sync::Arc;

use crate::injection_context::InjectionContext;
use crate::{Catalog, InjectionError};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// DependencySpec
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Specifies a particular way of resolving a dependency using the [`Catalog`]
pub trait DependencySpec {
    type IfaceType: ?Sized;
    type ReturnType;

    /// Resolve and create instances
    fn get(cat: &Catalog, ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// OneOf
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
    type IfaceType = Iface;
    type ReturnType = Arc<Iface>;

    fn get(cat: &Catalog, ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError> {
        let mut builders = cat.builders_for::<Iface>();
        if let Some(first) = builders.next() {
            if builders.next().is_some() {
                Err(InjectionError::ambiguous::<Iface>(ctx))
            } else {
                first.get_with_context(cat, ctx)
            }
        } else {
            Err(InjectionError::unregistered::<Iface>(ctx))
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// AllOf
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

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
    type IfaceType = Iface;
    type ReturnType = Vec<Arc<Iface>>;

    fn get(cat: &Catalog, ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError> {
        cat.builders_for::<Iface>()
            .map(|b| b.get_with_context(cat, ctx))
            .collect()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Maybe
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Returns `None` if an optional dependency is not registered
pub struct Maybe<Inner: DependencySpec + 'static> {
    _dummy: PhantomData<Inner>,
}

impl<Inner: DependencySpec + 'static> DependencySpec for Maybe<Inner> {
    type IfaceType = Inner::IfaceType;
    type ReturnType = Option<Inner::ReturnType>;

    fn get(cat: &Catalog, ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError> {
        match Inner::get(cat, ctx) {
            Ok(v) => Ok(Some(v)),
            Err(InjectionError::Unregistered(_)) => Ok(None),
            Err(err) => Err(err),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Lazy
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Delays the instantiation of a component until explicitly requested.
///
/// See [`crate::lazy::Lazy`] documentation for details.
pub struct Lazy<Inner: DependencySpec + 'static> {
    _dummy: PhantomData<Inner>,
}

impl<Inner: DependencySpec + 'static> DependencySpec for Lazy<Inner> {
    type IfaceType = Inner::IfaceType;
    type ReturnType = crate::lazy::Lazy<Inner::ReturnType>;

    #[cfg(not(feature = "tokio"))]
    fn get(cat: &Catalog, _ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError> {
        let cat = cat.clone();
        Ok(crate::lazy::Lazy::new(move || cat.get::<Inner>()))
    }

    #[cfg(feature = "tokio")]
    fn get(cat: &Catalog, _ctx: &InjectionContext) -> Result<Self::ReturnType, InjectionError> {
        // Lazy<T> will store the clone of a catalog it was initially created with
        // It will however first attempt to resolve a current catalog if scope feature
        // is used and only use the former as a fallback.
        let fallback_cat = cat.clone();
        Ok(crate::lazy::Lazy::new(
            move || match crate::catalog::CURRENT_CATALOG.try_with(|cat| cat.get::<Inner>()) {
                Ok(v) => v,
                Err(_) => fallback_cat.get::<Inner>(),
            },
        ))
    }
}
