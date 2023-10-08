//! Encapsules command line interface related implementations.

use clap::Parser;


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
    index: usize,
}

impl Iterator for CcWcArgsCommandIterator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let next = self.index + 1;
        if let Some(val) = self.command.0.rsplit(' ').nth(next) {
            self.index = next;
            Some(String::from(val))
        } else {
            None
        }
    }
}


// #[cfg(test)]
// mod tests {
//     // this is indirectly tested by our main tests.
// }
