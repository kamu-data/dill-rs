use std::{
    any::Any,
    sync::{Arc, Mutex},
};

/////////////////////////////////////////////////////////////////////////////////////////

pub trait Scope {
    fn get(&self) -> Option<Arc<dyn Any + Send + Sync>>;
    fn set(&self, inst: Arc<dyn Any + Send + Sync>);
}

/////////////////////////////////////////////////////////////////////////////////////////
// Transient
/////////////////////////////////////////////////////////////////////////////////////////

pub struct Transient;

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

/////////////////////////////////////////////////////////////////////////////////////////
// Singleton
/////////////////////////////////////////////////////////////////////////////////////////

pub struct Singleton {
    instance: Mutex<Option<Arc<dyn Any + Send + Sync>>>,
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
