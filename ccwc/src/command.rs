//! Encapsules command line interface related implementations.

use clap::Parser;

/// Prints line-, word-, and byte-count for every FILE, and one line with the total count, in case
/// of more than one FILE is provided. Without FILE, or in case if FILE is "-", input will be read
/// from standard input. One word is a series of non-empty characters, which are separated by
/// spaces.
#[derive(Debug, Eq, PartialEq, Parser)]
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
    pub file: String,
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
        assert_eq!(
            args,
            CcWcArgs {
                bytes: false,
                chars: false,
                lines: false,
                words: false,
                file: String::from("test.txt"),
            }
        );
    }

    #[test]
    fn args_from_flags() {
        let args = CcWcArgs::from("ccwc -w test.txt");
        assert_eq!(
            args,
            CcWcArgs {
                bytes: false,
                chars: false,
                lines: false,
                words: true,
                file: String::from("test.txt"),
            }
        );

        let args = CcWcArgs::from("ccwc -l test.txt");
        assert_eq!(
            args,
            CcWcArgs {
                bytes: false,
                chars: false,
                lines: true,
                words: false,
                file: String::from("test.txt"),
            }
        );

        let args = CcWcArgs::from("ccwc -cw test.txt");
        assert_eq!(
            args,
            CcWcArgs {
                bytes: true,
                chars: false,
                lines: false,
                words: true,
                file: String::from("test.txt"),
            }
        );
    }
}
