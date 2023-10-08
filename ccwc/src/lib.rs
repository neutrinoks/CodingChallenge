//! Coding challenge: Own version of word count (wc).

pub mod iterators;


use clap::Parser;
use std::{fs, error, str};


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


/// Checks if next character in iterator is equal to c, without modifying it.
fn check_if_next_is(chars: &str::Chars, c: char) -> bool {
    let mut cpy = chars.clone();
    cpy.next() == Some(c)
}


fn lines_chars(content: &str) -> (usize, usize) {
    let mut lines = 0;
    let mut chars = 0;
    let mut iter = content.chars();
    let mut next = iter.next();
    while let Some(c) = next {
        if c != '\n' {
            chars += 1;
        }
        if c == '\n' && !check_if_next_is(&iter, '\n') {
            lines += 1;
        }
        next = iter.next();
    }
    (lines, chars)
}


fn bytes(content: &str) -> usize {
    content.len()
}


fn words(content: &str) -> usize {
    let list = iterators::WordIterator::new(content);
    list.count()
}


pub fn ccwc(args: &CcWcArgs) -> Result<String, Box<dyn error::Error>> {
    let content = fs::read_to_string(&args.file)?;

    let bytes = bytes(&content);
    let words = words(&content);
    let (lines, _chars) = lines_chars(&content);
    
    Ok(format!("{} {} {} {}", lines, words, bytes, args.file))
}


#[cfg(test)]
mod tests {
    use super::*;

    const TESTFILE: &str = "tst.txt"; // e
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
    fn fn_lines_chars() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let (lines, chars) = lines_chars(&content);
        // assert_eq!(lines, 7137);
        // assert_eq!(chars, 339120);
        assert_eq!(lines, 15);
        assert_eq!(chars, 535);
    }

    #[test]
    fn fn_bytes() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let bytes = bytes(&content);
        // assert_eq!(bytes, 341836);
        assert_eq!(bytes, 537);
    }

    #[test]
    fn fn_words() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let words = words(&content);
        // assert_eq!(words, 58159);
        assert_eq!(words, 94);
    }

    #[test]
    fn cc_step_1_test() {
        let result = ccwc_from("ccwc -c test.txt").expect("ccwc error");
        assert_eq!(result, String::from("341836 test.txt"));
    }

    #[test]
    fn cc_step_2_test() {
        let result = ccwc_from("ccwc -l test.txt").expect("ccwc error");
        assert_eq!(result, String::from("7137 test.txt"));
    }

    #[test]
    fn cc_step_3_test() {
        let result = ccwc_from("ccwc -w test.txt").expect("ccwc error");
        assert_eq!(result, String::from("58159 test.txt"));
    }

    #[test]
    fn cc_step_4_test() {
        let result = ccwc_from("ccwc -m test.txt").expect("ccwc error");
        assert_eq!(result, String::from("339120 test.txt"));
    }

    // #[test]
    // fn cc_step_5_test() {
    //     let result = ccwc_from("ccwc test.txt").expect("ccwc error");
    //     assert_eq!(result, String::from("7137   58159  341836 test.txt"));
    // }

    // #[test]
    // fn cc_final_step() {
    //     // execute bash: "cat test.txt | ccwc -l"
    //     assert_eq!(result, String::from("7137"));
    // }
}
