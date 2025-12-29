use std::any::Any;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use crate::InjectionError;
use crate::cache::Cache;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Controls the lifetime of an instance created by
/// [`Builders`][`crate::Builder`]
pub trait Scope {
    fn get_or_create<Clb>(
        &self,
        cat: &crate::Catalog,
        create_instance: Clb,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>
    where
        Clb: FnOnce() -> Result<Arc<dyn Any + Send + Sync>, InjectionError>;
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Transient
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Never caches so that every dependency resolution will result in a new
/// instance.
pub struct Transient;

impl Default for Transient {
    fn default() -> Self {
        Self::new()
    }
}

impl Transient {
    pub fn new() -> Self {
        Self {}
    }
}

impl Scope for Transient {
    fn get_or_create<Clb>(
        &self,
        _cat: &crate::Catalog,
        create_instance: Clb,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>
    where
        Clb: FnOnce() -> Result<Arc<dyn Any + Send + Sync>, InjectionError>,
    {
        create_instance()
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Singleton
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Caches an instance upon first creation for the entire duration of the
/// program.
pub struct Singleton {
    instance: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
}

impl Default for Singleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Singleton {
    pub fn new() -> Self {
        Self {
            instance: Mutex::new(None),
        }
    }
}

impl Scope for Singleton {
    fn get_or_create<Clb>(
        &self,
        _cat: &crate::Catalog,
        create_instance: Clb,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>
    where
        Clb: FnOnce() -> Result<Arc<dyn Any + Send + Sync>, InjectionError>,
    {
        let mut cached = self.instance.lock().unwrap();
        if let Some(inst) = cached.as_ref() {
            Ok(inst.clone())
        } else {
            let inst = create_instance()?;
            *cached = Some(inst.clone());
            Ok(inst)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Cached
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Caches instances inside the specified [`Cache`] object. See [`Transaction`]
/// for common use case and examples.
pub struct Cached<T: Cache> {
    _ph: PhantomData<T>,
}

impl<T: Cache> Cached<T> {
    pub fn new() -> Self {
        Self {
            _ph: PhantomData::default(),
        }
    }
}

impl<T: Cache> Scope for Cached<T> {
    fn get_or_create<Clb>(
        &self,
        cat: &crate::Catalog,
        create_instance: Clb,
    ) -> Result<Arc<dyn Any + Send + Sync>, InjectionError>
    where
        Clb: FnOnce() -> Result<Arc<dyn Any + Send + Sync>, InjectionError>,
    {
        let id = self as *const Self as usize;
        let cache = cat.get_one::<T>()?;

        if let Some(inst) = cache.get(id) {
            Ok(inst.clone())
        } else {
            let inst = create_instance()?;
            cache.set(id, inst.clone());
            Ok(inst)
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// Transaction
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// A scope that caches instances within a current transaction. Transaction
/// boundaries are defined by adding [`TransactionCache`] instance into
/// a chained catalog.
///
/// Example use:
/// ```
/// #[dill::component]
/// #[dill::scope(dill::scopes::Transaction)]
/// struct Foo {}
///
/// // Init your base catalog
/// let base_catalog = dill::Catalog::builder()
///     .add::<Foo>()
///     .build();
///
/// // In some middleware (e.g. axum)
/// {
///     // Create chained catalog for duration of the transaction
///     // and add `TrasactionCache` to it to hold cached instances
///     let tx_catalog = base_catalog
///         .builder_chained()
///         .add_value(dill::scopes::TransactionCache::new())
///         .build();
///
///     // Further in code inject `Foo` somewhere
///     let foo1 = tx_catalog.get_one::<Foo>().unwrap();
///
///     // Next time you inhect `Foo` - you'll get the same instance
///     let foo2 = tx_catalog.get_one::<Foo>().unwrap();
///     assert_eq!(
///         foo1.as_ref() as *const Foo,
///         foo2.as_ref() as *const Foo,
///     );
///
///     // When chained catalog is dropped - the cache is dropped along with it
/// }
/// ```
pub type Transaction = Cached<TransactionCache>;

/// Just a newtype wrapper for [`CacheImpl`] to give it a specific type
/// identity. Used by [`Transaction`] scope.
pub struct TransactionCache(crate::cache::CacheImpl);

impl TransactionCache {
    pub fn new() -> Self {
        Self(crate::cache::CacheImpl::new())
    }
}

impl Cache for TransactionCache {
    #[inline(always)]
    fn get(&self, id: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.0.get(id)
    }

    #[inline(always)]
    fn set(&self, id: usize, inst: Arc<dyn Any + Send + Sync>) {
        self.0.set(id, inst)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
