use std::any::Any;
use std::sync::{Arc, Mutex};

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Controls the lifetime of an instance created by
/// [`Builders`][`crate::Builder`]
pub trait Scope {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    fn set(&self, inst: Arc<dyn Any + Send + Sync>);
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
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        None
    }

    fn set(&self, _inst: Arc<dyn Any + Send + Sync>) {}
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
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        self.instance.lock().unwrap().clone()
    }

    fn set(&self, inst: Arc<dyn Any + Send + Sync>) {
        self.instance.lock().unwrap().replace(inst);
    }
}
