//! Coding challenge: Own version of word count (wc).

pub mod command;
pub mod iterators;

use std::{error, str};

pub use command::{CcWcArgs, CcWcInput, Content};

/// Common Result type definition.
pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Checks if next character in iterator is equal to c, without modifying it.
fn check_next_is(chars: &str::Chars, c: char) -> bool {
    let mut cpy = chars.clone();
    Some(c) == cpy.next()
}

fn count_lines(piece: &str) -> usize {
    let mut lines = 0;
    let mut iter = piece.chars();
    while let Some(c) = iter.next() {
        if c == '\n' && !check_next_is(&iter, '\n') {
            lines += 1;
        }
    }
    lines
}

fn count_chars(piece: &str) -> usize {
    piece.char_indices().count()
}

fn count_bytes(piece: &str) -> usize {
    piece.as_bytes().len()
}

fn count_words(piece: &str) -> usize {
    iterators::WordIterator::new(piece).count()
}

fn iterate_pieces(content: &mut Content, f: fn(&str) -> usize) -> Result<usize> {
    let mut cnt: usize = 0;
    for piece in &mut *content {
        cnt += f(&piece);
    }
    content.rewind()?;
    Ok(cnt)
}

/// Main count function for lines in text.
pub fn lines(content: &mut Content) -> Result<usize> {
    iterate_pieces(content, count_lines)
}

/// Main count function for characters in text.
pub fn chars(content: &mut Content) -> Result<usize> {
    iterate_pieces(content, count_chars)
}

/// Main count function for number of bytes of this text.
pub fn bytes(content: &mut Content) -> Result<usize> {
    iterate_pieces(content, count_bytes)
}

/// Main count function for number of words in text.
pub fn words(content: &mut Content) -> Result<usize> {
    iterate_pieces(content, count_words)
}

/// Formats output for cli.
fn format_output(dvec: &Vec<usize>, digits: usize) -> String {
    match dvec.len() {
        1 => format!("{:>digit$}", dvec[0], digit = digits),
        2 => format!("{:>digit$} {:>digit$}", dvec[0], dvec[1], digit = digits),
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
pub fn ccwc(input: &mut command::CcWcInput) -> Result<String> {
    let no_flags = !(input.args.chars || input.args.bytes || input.args.words || input.args.lines);

    let mut dvec: Vec<usize> = Vec::new();
    if no_flags || input.args.lines {
        dvec.push(lines(&mut input.content)?);
    }
    if no_flags || input.args.words {
        dvec.push(words(&mut input.content)?);
    }
    if no_flags || input.args.bytes {
        dvec.push(bytes(&mut input.content)?);
    }
    if input.args.chars {
        dvec.push(chars(&mut input.content)?);
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
    use totems::assert_ok;

    const TESTFILE: &str = "test.txt";
    const TESTFILE_MISSING: &str = "could not open default test file";

    #[test]
    fn fn_lines() {
        let mut content = Content::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let lines = lines(&mut content);
        assert_ok!(lines, value == 7145);
    }

    #[test]
    fn fn_chars() {
        let mut content = Content::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let chars = chars(&mut content);
        assert_ok!(chars, value == 339292);
    }

    #[test]
    fn fn_bytes() {
        let mut content = Content::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let bytes = bytes(&mut content);
        assert_ok!(bytes, value == 342190);
    }

    #[test]
    fn fn_words() {
        let mut content = Content::read_to_string(TESTFILE).expect(TESTFILE_MISSING);
        let words = words(&mut content);
        assert_ok!(words, value == 58164);
    }

    #[test]
    fn cc_step_1_test() {
        let mut input = CcWcInput::try_from("ccwc -c test.txt").unwrap();
        let result = ccwc(&mut input).expect("ccwc error");
        assert_eq!(result, String::from("342190 test.txt"));
    }

    #[test]
    fn cc_step_2_test() {
        let mut input = CcWcInput::try_from("ccwc -l test.txt").unwrap();
        let result = ccwc(&mut input).expect("ccwc error");
        assert_eq!(result, String::from("7145 test.txt"));
    }

    #[test]
    fn cc_step_3_test() {
        let mut input = CcWcInput::try_from("ccwc -w test.txt").unwrap();
        let result = ccwc(&mut input).expect("ccwc error");
        assert_eq!(result, String::from("58164 test.txt"));
    }

    #[test]
    fn cc_step_4_test() {
        let mut input = CcWcInput::try_from("ccwc -m test.txt").unwrap();
        let result = ccwc(&mut input).expect("ccwc error");
        assert_eq!(result, String::from("339292 test.txt"));
    }

    #[test]
    fn cc_step_5_test() {
        let mut input = CcWcInput::try_from("ccwc test.txt").unwrap();
        let result = ccwc(&mut input).expect("ccwc error");
        assert_eq!(result, String::from("  7145  58164 342190 test.txt"));
    }

    // Integration test, manually via shell...
    // #[test]
    // fn cc_final_step() {
    //     // execute bash: "cat test.txt | ccwc -l"
    //     assert_eq!(b"7145\n", output.stdout.as_slice());
    // }
}
