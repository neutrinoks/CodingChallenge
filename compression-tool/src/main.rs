//! Just a main for to create a binary out of this...

use compression_tool::command::CtInput;

fn main() -> compression_tool::Result<()> {
    let args = CtInput::parse_input()?;
    let cli_out = compression_tool::compression_tool(args)?;
    println!("{}", cli_out);
    Ok(())
}
