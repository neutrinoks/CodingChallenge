//! Coding challenge: Own version of word count (wc).

pub mod command;
pub mod iterators;

use std::{error, str};

pub use command::{CcWcArgs, CcWcInput};

/// Checks if next character in iterator is equal to c, without modifying it.
fn check_next_is(chars: &str::Chars, c: char) -> bool {
    let mut cpy = chars.clone();
    Some(c) == cpy.next()
}

/// Main count function for lines in text.
pub fn lines(content: &str) -> usize {
    let mut lines = 0;
    let mut iter = content.chars();
    while let Some(c) = iter.next() {
        if c == '\n' && !check_next_is(&iter, '\n') {
            lines += 1;
        }
    }
    lines
}

/// Main count function for characters in text.
pub fn chars(content: &str) -> usize {
    content.char_indices().count()
}

/// Main count function for number of bytes of this text.
#[inline]
pub fn bytes(content: &str) -> usize {
    content.as_bytes().len()
}

/// Main count function for number of words in text.
#[inline]
pub fn words(content: &str) -> usize {
    iterators::WordIterator::new(content).count()
}

/// Formats output for cli.
fn format_output(dvec: &Vec<usize>, digits: usize) -> String {
    match dvec.len() {
        1 => format!("{:>digit$}", dvec[0], digit = digits),
        2 => format!(
            "{:>digit$} {:>digit$}",
            dvec[0],
            dvec[1],
            digit = digits
        ),
        3 => format!(
            "{:>digit$} {:>digit$} {:>digit$}",
            dvec[0],
            dvec[1],
            dvec[2],
            digit = digits
        ),
        4 => format!(
            "{:>digit$} {:>digit$} {:>digit$} {:>digit$}",
            dvec[0],
            dvec[1],
            dvec[2],
            dvec[3],
            digit = digits
        ),
        _ => panic!("number of outputs not supported"),
    }
}

/// This is the main entry function for ccwc.
pub fn ccwc(input: &command::CcWcInput) -> Result<String, Box<dyn error::Error>> {
    let no_flags = !(input.args.chars || input.args.bytes || input.args.words || input.args.lines);

    let mut dvec: Vec<usize> = Vec::new();
    if no_flags || input.args.lines {
        dvec.push(lines(&input.content));
    }
    if no_flags || input.args.words {
        dvec.push(words(&input.content));
    }
    if no_flags || input.args.bytes {
        dvec.push(bytes(&input.content));
    }
    if input.args.chars {
        dvec.push(chars(&input.content));
    }
    let digits = dvec.iter().max().unwrap().to_string().len();
    
    let mut output = format_output(&dvec, digits);
    if let Some(file) = &input.args.file {
        output.push(' ');
        output.push_str(file);
    }
    Ok(output)
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
        assert_eq!(result, String::from("342190 test.txt"));
    }

    #[test]
    fn cc_step_2_test() {
        let args = CcWcArgs::from("ccwc -l test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("7145 test.txt"));
    }

    #[test]
    fn cc_step_3_test() {
        let args = CcWcArgs::from("ccwc -w test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("58164 test.txt"));
    }

    #[test]
    fn cc_step_4_test() {
        let args = CcWcArgs::from("ccwc -m test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("339292 test.txt"));
    }

    #[test]
    fn cc_step_5_test() {
        let args = CcWcArgs::from("ccwc test.txt");
        let result = ccwc(&args).expect("ccwc error");
        assert_eq!(result, String::from("  7145  58164 342190 test.txt"));
    }

    #[test]
    fn cc_final_step() {
        // execute bash: "cat test.txt | ccwc -l"
        let output = std::process::Command::new("cat test.txt | ccwc -l")
            .arg("Hello world")
            .output()
            .expect("Failed to execute command");
        assert_eq!(b"7137\n", output.stdout.as_slice());
    }
}
