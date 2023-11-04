//! Encapsules command line interface related implementations.

use clap::Parser;
use std::{
    error, fs,
    io::{self, BufReader, IsTerminal, Read, Seek},
};

/// This threshold affects whether a file will be read in completely or iterated vai buffer.
const FILE_SIZE_THRESHOLD: usize = 10_000_000;

/// Content management system for providing either the full content as String, or in case of larger
/// files piece by piece.
#[derive(Debug)]
pub enum Content {
    /// Small file, we read in the full content.
    SmallFile(String, bool),
    /// Large file, we read the content piece by piece.
    LargeFile(BufReader<fs::File>),
}

impl Content {
    /// Renews the iterator, because it will be consumed multiple times.
    pub fn rewind(&mut self) -> crate::Result<()> {
        match self {
            Content::SmallFile(_, flag) => *flag = true,
            Content::LargeFile(reader) => reader.rewind()?,
        }
        Ok(())
    }

    /// Pendant-method to fs::read_to_string().
    pub fn read_to_string(file: &str) -> crate::Result<Content> {
        let file_size = fs::metadata(file)?.len() as usize;
        if file_size > FILE_SIZE_THRESHOLD {
            let file = fs::File::open(file)?;
            let reader = BufReader::new(file);
            Ok(Content::LargeFile(reader))
        } else {
            Ok(Content::SmallFile(fs::read_to_string(file)?, true))
        }
    }
}

impl Iterator for Content {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        match self {
            Content::SmallFile(content, flag) => {
                if *flag {
                    *flag = false;
                    Some(content.clone())
                } else {
                    None
                }
            }
            Content::LargeFile(reader) => {
                let mut content = String::new();
                unsafe {
                    if reader.read(content.as_bytes_mut()).is_ok() {
                        Some(content)
                    } else {
                        None
                    }
                }
            }
        }
    }
}

/// The whole input data for main function (parameters and text to be processed).
#[derive(Debug)]
pub struct CcWcInput {
    /// CLI parameters.
    pub args: CcWcArgs,
    /// Content to be analyzed.
    pub content: Content,
}

impl CcWcInput {
    /// Default method to process user input from command line. Method checks whether stdin was used to
    /// path a text to be analyzed or a filename was passed to be read in.
    pub fn parse_input() -> crate::Result<CcWcInput> {
        let (args, content) = if io::stdin().is_terminal() {
            // No usage of stdin, a filename should be provided.
            let args = CcWcArgs::parse();
            let content = if let Some(file) = &args.file {
                // Check file size and decide for reading in completely or buffered.
                Content::read_to_string(file)?
            } else {
                return Err(String::from("No input file or data was provided").into());
            };
            (args, content)
        } else {
            // Stdin provides content input, no filename should be provided.
            let mut content = String::new();
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
            (args, Content::SmallFile(content, true))
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
        let content = Content::SmallFile(fs::read_to_string(args.file.as_ref().unwrap())?, true);
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
struct CcWcArgsCommand<'r>(&'r str);

impl<'r> From<&'r str> for CcWcArgsCommand<'r> {
    fn from(input: &'r str) -> CcWcArgsCommand<'r> {
        CcWcArgsCommand(input)
    }
}

impl<'r> IntoIterator for CcWcArgsCommand<'r> {
    type Item = &'r str;
    type IntoIter = std::str::Split<'r, char>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.split(' ')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arg_iter_test() {
        let cmd = CcWcArgsCommand("ccwc -c test.txt");
        let mut iter = cmd.into_iter();
        assert_eq!(iter.next(), Some("ccwc"));
        assert_eq!(iter.next(), Some("-c"));
        assert_eq!(iter.next(), Some("test.txt"));
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
