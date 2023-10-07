//! An own count words version (cw).

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ccwc::CcWcArgs::parse();
    let cli_out = ccwc::ccwc(&args)?;
    println!("{cli_out}");
    Ok(())
}
