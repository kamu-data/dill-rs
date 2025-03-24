mod add;
mod list;

pub use add::*;
pub use list::*;

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    async fn run(&self) -> std::io::Result<()>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CommandDesc {
    pub needs_transaction: bool,
}
