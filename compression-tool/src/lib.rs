//! Library with functionality of compression-tool.

pub mod command;

use std::collections::HashMap;
use command::CtInput;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Stores a single frequency-bin, e.g. for the character 'a', how many times 'a' appeared in a 
/// given input stream.
#[derive(Debug, Eq, PartialEq)]
pub struct CharSpectrum(Vec<(char, usize)>);

impl CharSpectrum {
    pub fn new() -> CharSpectrum {
        CharSpectrum(Vec::new())
    }

    pub fn from_stream(stream: &str) -> CharSpectrum {
        let mut s = CharSpectrum::new();
        s.analyse_stream(stream);
        s
    }

    pub fn analyse_stream(&mut self, stream: &str) {
        let mut map: HashMap<char, usize> = HashMap::new();
        stream.chars().for_each(|c| {
            let cnt = if let Some(cnt) = map.get(&c) {
                cnt + 1
            } else {
                1
            };
            let _ = map.insert(c, cnt);
        });
        self.0 = map.into_iter().collect();
        self.0.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }
}

/// TODO
pub fn frequency_analysis(input: &CtInput) -> Result<CharSpectrum> {
    let mut spectrum = CharSpectrum::new();
    spectrum.analyse_stream(&input.content);
    Ok(spectrum)
}

/// Main entry method for compression-tool use case, to be able to separate the code into library
/// and not main module.
pub fn compression_tool(input: CtInput) -> Result<String> {
    let result = frequency_analysis(&input)?;
    Ok(format!("{result:?}"))
}

#[cfg(test)]
pub fn testfile(name: &str) -> CtInput {
    let args = crate::command::CtArgs{ filename: name.to_string() };
    CtInput::try_from(args).expect(&format!("testfile/expected: {}", name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_1() {
        let result = frequency_analysis(&testfile("135-0.txt")).expect("frequency_analysis failed");
        let t = result.0.iter().find(|&&x| x.0 == 't').expect("no 't' found");
        let x = result.0.iter().find(|&&x| x.0 == 'X').expect("no 'X' found");
        assert_eq!(t.1, 223000);
        assert_eq!(x.1, 333);
    }

    #[test]
    fn step_2() {
        todo!();
    }

    #[test]
    fn step_3() {
        todo!();
    }

    #[test]
    fn step_4() {
        todo!();
    }

    #[test]
    fn step_5() {
        todo!();
    }
}
