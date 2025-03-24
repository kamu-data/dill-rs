#[derive(Debug, clap::Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Add(Add),
    List(List),
}

#[derive(Debug, Clone, clap::Args)]
pub struct Add {
    #[arg(long)]
    pub value: i32,
}

#[derive(Debug, Clone, clap::Args)]
pub struct List {
    #[arg(short, long)]
    pub verbose: bool,
}
