use std::path::PathBuf;
use std::sync::Arc;

use super::Transaction;
use crate::domain::ValueRepo;

#[derive(Debug, Clone)]
pub struct ValueRepoPath(pub PathBuf);

#[dill::component]
#[dill::interface(dyn ValueRepo)]
pub struct ValueRepoImpl {
    path: ValueRepoPath,

    #[expect(unused)]
    tx: Arc<Transaction>,
}

impl ValueRepo for ValueRepoImpl {
    fn get(&self) -> std::io::Result<i32> {
        match std::fs::read_to_string(&self.path.0) {
            Ok(s) => Ok(s.parse().ok().unwrap_or_default()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
            Err(e) => Err(e),
        }
    }

    fn set(&self, value: i32) -> std::io::Result<()> {
        std::fs::write(&self.path.0, value.to_string())?;
        Ok(())
    }
}
