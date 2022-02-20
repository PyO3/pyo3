use structopt::StructOpt;

pub mod cli;
use cli::Subcommand;

pub mod pytests;
pub mod llvm_cov;
pub mod utils;

fn main() -> anyhow::Result<()> {
    Subcommand::from_args().execute()
}
