//! Coding challenge: Own version of word count (wc).

use clap::Parser;
use std::{fs, error};


/// Prints line-, word-, and byte-count for every FILE, and one line with the total count, in case
/// of more than one FILE is provided. Without FILE, or in case if FILE is "-", input will be read
/// from standard input. One word is a series of non-empty characters, which are separated by
/// spaces.
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct CcWcArgs {
    /// Outputs the number of bytes.
    #[clap(short, long, action)]
    pub bytes: bool,
    /// Outputs the number of lines.
    #[clap(short, long, action)]
    pub chars: bool,
    /// Outputs the number of words.
    #[clap(short, long, action)]
    pub lines: bool,
    /// Filename of file to be counted.
    pub file: String,
}


// fn check_next() ->  {
// }


fn chars_lines(content: &str) -> (usize, usize) {
    let mut lines = 0;
    let mut chars = 0;
    content.chars().for_each(|x| {
        if x.is_alphanumeric() {
            chars += 1;
        }
        if x == '\n' {
            lines += 1;
        }
    });
    (lines, chars)
}


fn bytes(content: &str) -> usize {
    content.len()
}


fn words(content: &str) -> usize {
    content.rsplit(' ').count()
}


pub fn ccwc(args: &CcWcArgs) -> Result<String, Box<dyn error::Error>> {
    Ok(String::new())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        ffi::OsString,
        str::{Chars, FromStr},
        iter::IntoIterator,
    };

    const TESTFILE: &str = "test.txt";
    const TESTFILE_MISSING: &str = "could not open default test file";

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


    fn ccwc_from(input: &str) -> Result<String, Box<dyn error::Error>> {
        let args = CcWcArgs::parse_from(CcWcArgsCommand::from(input));
        ccwc(&args)
    }


    #[test]
    fn fn_chars_lines() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let (lines, chars) = chars_lines(&content);
        assert_eq!(lines, 7137);
        assert_eq!(chars, 339120);
    }

    #[test]
    fn fn_bytes() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let bytes = bytes(&content);
        assert_eq!(bytes, 341836);
    }

    #[test]
    fn fn_words() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let words = words(&content);
        assert_eq!(words, 58159);
    }

    #[test]
    fn cc_step_1_test() {
        let result = ccwc_from("ccwc -c test.txt").expect("ccwc error");
        assert_eq!(result, String::from("341836 test.txt"));
    }

    #[test]
    fn cc_step_2_test() {
        todo!();
    }

    #[test]
    fn cc_step_3_test() {
        todo!();
    }

    #[test]
    fn cc_step_4_test() {
        todo!();
    }

    #[test]
    fn cc_step_5_test() {
        todo!();
    }
}
