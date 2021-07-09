use std::{marker::PhantomData, sync::Arc};

use crate::{Catalog, InjectionError};

/////////////////////////////////////////////////////////////////////////////////////////
// DependencySpec
/////////////////////////////////////////////////////////////////////////////////////////

pub trait DependencySpec {
    type ReturnType;
    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError>;
}

/////////////////////////////////////////////////////////////////////////////////////////
// OneOf
/////////////////////////////////////////////////////////////////////////////////////////

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

    fn get(cat: &Catalog) -> Result<Self::ReturnType, InjectionError> {
        let mut builders = cat.builders_for::<Iface>();
        if let Some(first) = builders.next() {
            if builders.next().is_some() {
                Err(InjectionError::Ambiguous)
            } else {
                first.get(cat)
            }
        } else {
            Err(InjectionError::unregistered::<Iface>())
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////////
// AllOf
/////////////////////////////////////////////////////////////////////////////////////////

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
}
