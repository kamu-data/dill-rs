use std::sync::Arc;

use super::CommandDesc;
use crate::commands::Command;
use crate::domain::ValueRepo;

pub struct AddCommand {
    repo: Arc<dyn ValueRepo>,
    value: i32,
}

#[dill::component(pub)]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc { needs_transaction: true })]
impl AddCommand {
    pub fn new(repo: Arc<dyn ValueRepo>, #[dill::component(explicit)] value: i32) -> Self {
        Self { repo, value }
    }
}

#[async_trait::async_trait]
impl Command for AddCommand {
    async fn run(&self) -> std::io::Result<()> {
        let old = self.repo.get()?;
        let new = old + self.value;
        self.repo.set(new)?;

        eprintln!("{} add {} equals {}", old, self.value, new);
        Ok(())
    }
}
