//! Coding challenge: Own version of word count (wc).

pub mod command;
pub mod iterators;

use std::{error, fs, str};

pub use command::CcWcArgs;


/// Checks if next character in iterator is equal to c, without modifying it.
fn check_if_next_is(chars: &str::Chars, c: char) -> bool {
    let mut cpy = chars.clone();
    Some(c) == cpy.next()
}

pub fn lines(content: &str) -> usize {
    let mut lines = 0;
    let mut iter = content.chars();
    while let Some(c) = iter.next() {
        if c == '\n' && !check_if_next_is(&iter, '\n') {
            lines += 1;
        }
    }
    lines
}

pub fn chars(content: &str) -> usize {
    let mut chars = 0;
    let iter = content.char_indices();
    for (_, c) in iter {
        if c.is_ascii() {
            chars += 1;
        }
    }
    chars
}

#[inline]
pub fn bytes(content: &str) -> usize {
    content.as_bytes().len()
}

#[inline]
pub fn words(content: &str) -> usize {
    iterators::WordIterator::new(content).count()
}


/// This is the main entry function for ccwc.
pub fn ccwc(args: &command::CcWcArgs) -> Result<String, Box<dyn error::Error>> {
    let content = fs::read_to_string(&args.file)?;

    let bytes = bytes(&content);
    let words = words(&content);
    let lines = lines(&content);

    let output = vec![lines, words, bytes];
    let digits = output.iter().max().unwrap().to_string().len();

    Ok(format!("{:>digit$} {:>digit$} {:>digit$} {}", lines, words, bytes, args.file, digit=digits))
}


#[cfg(test)]
mod tests {
    use super::*;

    const TESTFILE: &str = "test.txt";
    const TESTFILE_MISSING: &str = "could not open default test file";

    #[test]
    fn fn_lines() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let lines = lines(&content);
        assert_eq!(lines, 7145);
    }

    #[test]
    fn fn_chars() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let chars = chars(&content);
        assert_eq!(chars, 339292);
    }

    #[test]
    fn fn_bytes() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let bytes = bytes(&content);
        assert_eq!(bytes, 342190);
    }

    #[test]
    fn fn_words() {
        let content = fs::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let words = words(&content);
        assert_eq!(words, 58164);
    }

    #[test]
    fn cc_step_1_test() {
        let args = CcWcArgs::from("ccwc -c test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("341836 test.txt"));
    }

    #[test]
    fn cc_step_2_test() {
        let args = CcWcArgs::from("ccwc -l test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("7137 test.txt"));
    }

    #[test]
    fn cc_step_3_test() {
        let args = CcWcArgs::from("ccwc -w test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("58159 test.txt"));
    }

    #[test]
    fn cc_step_4_test() {
        let args = CcWcArgs::from("ccwc -m test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("339120 test.txt"));
    }

    // #[test]
    // fn cc_step_5_test() {
    //     let args = CcWcArgs::from();
    //     let result = ccwc_from("ccwc test.txt").expect("ccwc error");
    //     assert_eq!(result, String::from("7137   58159  341836 test.txt"));
    // }

    // #[test]
    // fn cc_final_step() {
    //     // execute bash: "cat test.txt | ccwc -l"
    //     assert_eq!(result, String::from("7137"));
    // }
}
