//! Library with functionality of compression-tool.

pub mod command;

use std::collections::HashMap;
use command::CtInput;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Stores a single frequency-bin, e.g. for the character 'a', how many times 'a' appeared in a 
/// given input stream.
#[derive(Debug, Eq, PartialEq)]
pub struct CharSpectrum(HashMap<char, usize>);

impl CharSpectrum {
    pub fn new() -> CharSpectrum {
        CharSpectrum(HashMap::new())
    }

    pub fn analyse_stream(&mut self, stream: &str) {
        todo!();
    }

    pub fn count(&mut self, c: char) {
        let cnt = if let Some(cnt) = self.0.get(&c) {
            cnt + 1
        } else {
            0
        };
        let _ = self.0.insert(c, cnt);
    }
}

/// TODO
pub fn frequency_analysis(input: &CtInput) -> Result<()> {
    Ok(())
}

/// Main entry method for compression-tool use case, to be able to separate the code into library
/// and not main module.
pub fn compression_tool(input: CtInput) -> Result<String> {
    Ok(input.content)
}
