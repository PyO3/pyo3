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

    cli::Subcommand::from_args().execute()?;
    Ok(())
}
