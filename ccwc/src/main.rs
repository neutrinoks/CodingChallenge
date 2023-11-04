//! An own count words version (cw).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = ccwc::CcWcInput::parse_input()?;
    let cli_out = ccwc::ccwc(&mut args)?;
    println!("{cli_out}");
    Ok(())
}
