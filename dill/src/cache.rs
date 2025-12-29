use std::any::Any;
use std::sync::Arc;

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait Cache: Send + Sync + 'static {
    fn get(&self, id: usize) -> Option<Arc<dyn Any + Send + Sync>>;
    fn set(&self, id: usize, inst: Arc<dyn Any + Send + Sync>);
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub struct CacheImpl {
    slots: Arc<std::sync::RwLock<std::collections::BTreeMap<usize, Arc<dyn Any + Send + Sync>>>>,
}

impl CacheImpl {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
        }
    }
}

impl Cache for CacheImpl {
    fn get(&self, id: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.slots.read().unwrap().get(&id).cloned()
    }

    fn set(&self, id: usize, inst: Arc<dyn Any + Send + Sync>) {
        self.slots.write().unwrap().insert(id, inst);
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
