//! Just a main for to create a binary out of this...

use ccct::command::CtInput;

fn main() -> ccct::Result<()> {
    let args = CtInput::parse_input()?;
    let cli_out = ccct::compression_tool(args)?;
    println!("{}", cli_out);
    Ok(())
}
