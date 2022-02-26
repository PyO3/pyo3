use clap::ErrorKind::MissingArgumentOrSubcommand;
use structopt::StructOpt;

pub mod cli;
pub mod clippy;
pub mod doc;
pub mod fmt;
pub mod llvm_cov;
pub mod pytests;
pub mod test;
pub mod utils;

fn main() -> anyhow::Result<()> {
    utils::print_metadata()?;

    // Avoid spewing backtraces all over the command line
    // For some reason this is automatically enabled on nightly compilers...
    std::env::set_var("RUST_LIB_BACKTRACE", "0");

    match cli::Subcommand::from_args_safe() {
        Ok(c) => c.execute()?,
        Err(e) if e.kind == MissingArgumentOrSubcommand => cli::Subcommand::All.execute()?,
        Err(e) => return Err(e)?,
    }
    Ok(())
}
