//! An own count words version (cw).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ccwc::CcWcInput::parse_input()?;
    let cli_out = ccwc::ccwc(&args)?;
    println!("{cli_out}");
    Ok(())
}
