mod cli;
mod commands;
mod domain;
mod infra;

use clap::Parser as _;
use commands::{Command, CommandDesc};
use dill::{BuilderExt, Component as _, TypedBuilderCast as _};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = cli::Cli::parse();

    // Select the appropriate command builder based on CLI arguments.
    // Command builders use #[component(explitic)] attributes to mark values they
    // want to take directly from CLI arguments instead of injecting them.
    let command_builder: Box<dyn dill::TypedBuilder<dyn Command>> = match args.command {
        cli::Command::Add(add) => {
            // Notice how we are forced to pass explicit arguments to get a builder
            // This allows us to catch missing arguments at compile time instead of getting
            // DI errors in runtime
            Box::new(commands::AddCommand::builder(add.value).cast())
        }
        cli::Command::List(_list) => Box::new(commands::ListCommand::builder().cast()),
    };

    // Set up dependencies.
    // Here we could use command builder's metadata to determine how to set up
    // the catalog, e.g. whether a command requires opening a DB transaction,
    // or requires some authorization to be added to execute on behalf of some
    // user.
    let mut b = dill::Catalog::builder();
    b.add_value(infra::ValueRepoPath("./state.txt".into()))
        .add::<infra::ValueRepoImpl>();

    if command_builder
        .metadata_get_first::<CommandDesc>()
        .copied()
        .unwrap_or_default()
        .needs_transaction
    {
        b.add::<infra::Transaction>();
    }

    let catalog = b.build();

    // Finally we construct the command using the configured catalog to inject
    // dependencies.
    let command = command_builder.get(&catalog).unwrap();
    command.run().await
}
