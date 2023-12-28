//! Just a main for to create a binary out of this...

use cccompress::CtInput;

fn main() -> cccompress::Result<()> {
    let args = CtInput::parse_input()?;
    let cli_out = cccompress::compression_tool(args)?;
    println!("{}", cli_out);
    Ok(())
}
