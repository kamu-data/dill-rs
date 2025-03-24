use std::sync::Arc;

use super::CommandDesc;
use crate::commands::Command;
use crate::domain::ValueRepo;

#[dill::component]
#[dill::interface(dyn Command)]
#[dill::meta(CommandDesc { needs_transaction: true })]
pub struct ListCommand {
    repo: Arc<dyn ValueRepo>,
}

#[async_trait::async_trait]
impl Command for ListCommand {
    async fn run(&self) -> std::io::Result<()> {
        eprintln!("Value: {}", self.repo.get()?);
        Ok(())
    }
}
