//! Library with functionality of compression-tool.

pub mod command;

use command::CtInput;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


pub fn compression_tool(input: CtInput) -> Result<String> {
    Ok(input.content)
}
