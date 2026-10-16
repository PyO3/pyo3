use std::{env, fs};

use anyhow::{bail, ensure, Context, Result};
use object::{Object, ObjectSymbol};

fn main() -> Result<()> {
    const HELP: &str =
        "should be called with exactly one command line arg that is path of file to analyze";

    let mut args = env::args().skip(1);
    let path = args.next().context(HELP)?;
    ensure!(args.next().is_none(), "{HELP}");

    let file = fs::read(&path).with_context(|| format!("failed to read file: {path}"))?;
    let file = object::read::File::parse(file.as_slice())
        .with_context(|| format!("failed to parse file: {path}"))?;
    let mut count = 0;
    for symbol in file.symbols() {
        let name = symbol.name().context("failed to get name of a symbol")?;
        if name.contains("std") {
            count += 1;
            println!("found symbol from `std`: {name}");
        }
    }

    if count > 0 {
        bail!("\ncompiled library contains {count} symbols from std");
    }

    Ok(())
}
