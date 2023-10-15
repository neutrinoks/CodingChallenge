//! Encapsules command line interface related implementations.

use clap::Parser;
use std::{
    error, fs,
    io::{self, BufReader, IsTerminal, Read},
};

/// The whole input data for main function (parameters and text to be processed).
#[derive(Debug)]
pub struct CcWcInput {
    /// CLI parameters.
    pub args: CcWcArgs,
    /// Content to be analyzed.
    pub content: String,
}

impl CcWcInput {
    /// Default method to process user input from command line. Method checks whether stdin was used to
    /// path a text to be analyzed or a filename was passed to be read in.
    pub fn parse_input() -> Result<CcWcInput, Box<dyn error::Error>> {
        let mut content = String::new();
        let args = if io::stdin().is_terminal() {
            // No usage of stdin, a filename should be provided.
            let args = CcWcArgs::parse();
            if let Some(file) = &args.file {
                content = fs::read_to_string(file)?;
            } else {
                return Err(String::from("No input file or data was provided").into());
            }
            args
        } else {
            // Stdin provides content input, no filename should be provided.
            let mut reader = BufReader::new(io::stdin());
            reader.read_to_string(&mut content)?;
            let mut args = CcWcArgs::parse();
            if let Some(file) = args.file {
                println!(
                    "Warning: file `{}` will be ignored because stdin-input was provided",
                    file
                );
                args.file = None;
            }
            args
        };

        Ok(CcWcInput { args, content })
    }
}

impl TryFrom<&str> for CcWcInput {
    type Error = Box<dyn error::Error>;

    fn try_from(cmd: &str) -> Result<CcWcInput, Self::Error> {
        let args = CcWcArgs::parse_from(CcWcArgsCommand::from(cmd));
        if args.file.is_none() {
            return Err(io::Error::new(io::ErrorKind::Other, "no file has been specified").into());
        }
        let content = fs::read_to_string(args.file.as_ref().unwrap())?;
        Ok(CcWcInput { args, content })
    }
}

/// Prints line-, word-, and byte-count for every FILE, and one line with the total count, in case
/// of more than one FILE is provided. Without FILE, or in case if FILE is "-", input will be read
/// from standard input. One word is a series of non-empty characters, which are separated by
/// spaces.
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CcWcArgs {
    /// Outputs the number of bytes.
    #[clap(short('c'), long, action)]
    pub bytes: bool,
    /// Outputs the number of characters.
    #[clap(short('m'), long, action)]
    pub chars: bool,
    /// Outputs the number of lines.
    #[clap(short('l'), long, action)]
    pub lines: bool,
    /// Outputs the number of words.
    #[clap(short('w'), long, action)]
    pub words: bool,
    /// Filename of file to be counted.
    pub file: Option<String>,
}

impl From<&str> for CcWcArgs {
    fn from(cmd: &str) -> CcWcArgs {
        CcWcArgs::parse_from(CcWcArgsCommand::from(cmd))
    }
}

#[derive(Clone, Debug)]
struct CcWcArgsCommand(String);

impl From<&str> for CcWcArgsCommand {
    fn from(input: &str) -> CcWcArgsCommand {
        CcWcArgsCommand(String::from(input))
    }
}

impl IntoIterator for CcWcArgsCommand {
    type Item = String;
    type IntoIter = CcWcArgsCommandIterator;

    fn into_iter(self) -> Self::IntoIter {
        CcWcArgsCommandIterator {
            command: self,
            index: 0,
        }
    }
}

#[derive(Clone, Debug)]
struct CcWcArgsCommandIterator {
    command: CcWcArgsCommand,
    // iter: std::str::Split<'_>
    index: usize,
}

impl Iterator for CcWcArgsCommandIterator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if let Some(val) = self.command.0.split(' ').nth(self.index) {
            self.index += 1;
            Some(String::from(val))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_iter_test() {
        let cmd = CcWcArgsCommand(String::from("ccwc -c test.txt"));
        let mut iter = cmd.into_iter();
        assert_eq!(iter.next(), Some(String::from("ccwc")));
        assert_eq!(iter.next(), Some(String::from("-c")));
        assert_eq!(iter.next(), Some(String::from("test.txt")));
    }

    #[test]
    fn args_from_only_filename() {
        let args = CcWcArgs::from("ccwc test.txt");
        assert_eq!(args.bytes, false);
        assert_eq!(args.chars, false);
        assert_eq!(args.lines, false);
        assert_eq!(args.words, false);
        assert_eq!(args.file, Some(String::from("test.txt")));
    }

    #[test]
    fn args_from_flags() {
        let args = CcWcArgs::from("ccwc -w test.txt");
        assert_eq!(args.bytes, false);
        assert_eq!(args.chars, false);
        assert_eq!(args.lines, false);
        assert_eq!(args.words, true);

        let args = CcWcArgs::from("ccwc -l test.txt");
        assert_eq!(args.bytes, false);
        assert_eq!(args.chars, false);
        assert_eq!(args.lines, true);
        assert_eq!(args.words, false);

        let args = CcWcArgs::from("ccwc -cw test.txt");
        assert_eq!(args.bytes, true);
        assert_eq!(args.chars, false);
        assert_eq!(args.lines, false);
        assert_eq!(args.words, true);
    }
}
