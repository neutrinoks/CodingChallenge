//! Module contains the JSON-parser and the internal used JSON-Lexer. For more background
//! informations on the following definitions have a look at the (RFC-8259)[ 
//! https://www.rfc-editor.org/rfc/rfc8259].
//!
//! ### Some notes from the RFC-8259
//!
//! *Whitespaces* are: Space, horizontal tab, line feed or new line, CR
//!
//! *Values* are: Object, array, number, string, or true/false/null.
//!
//! *Objects* are: begin-object [ member (value-separator member)* ] end-object, where
//! member is: string name-separator value

use JLexerToken::*;

macro_rules! whitespace_pat {
    () => {
        ' ' | '\n' | '\r' | '\t'
    }
}

macro_rules! number_pat {
    () => {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.'
    }
}

/// All possible tokens provided by the JLexer.
#[derive(Debug, Clone, PartialEq)]
pub enum JLexerToken {
    /// One or multiple whitespaces of any kind (' ', '\r', '\n', '\t').
    Whitespace,
    /// '{'.
    ObjectBegin,
    /// '}'.
    ObjectEnd,
    /// '['.
    ArrayBegin,
    /// ']'.
    ArrayEnd,
    /// ','.
    NameSeparator,
    /// ':'.
    ValueSeparator,
    /// '"'.
    StringToken,
    /// 'true'.
    TrueToken,
    /// 'false'.
    FalseToken,
    /// 'null'.
    NullToken,
    /// Any kind of string content.
    StringContent(String),
    /// Integer number.
    NumberInteger(isize),
    /// Floating point value.
    NumberFloat(f64),
    /// Any other unknown token, which we are not able to identify.
    UnknownToken(String),
}

/// Our JSON-lexer to go through string based source.
///
/// Possible structural lexer-tokens:
/// - '[' as begin-array token
/// - '{' as begin-object token
/// - ']' as end-array token
/// - '}' as end-object token
/// - ':' as name-separator
/// - ',' as value-separator
///
/// Additional thoughts:
/// - Collect string-literals by '"'
/// - We do net decide whether a string is correct in terms of identifying names or regular
///   strings. We only do providing raw lexer tokens, it is the Parser's job to decide whether they 
///   are correct.
/// - We have different whitespaces, so we provide only one whitespace token for multiple in a row.
/// 
/// From this informations we derive the possible tokens, see JLexerToken.
pub struct JLexer<'s> {
    /// Internal iterator for string-based source.
    iter: std::str::CharIndices<'s>,
    /// Last tokens we identified. Last one: last_tk[1], before last one: last_tk[0].
    last_tk: [JLexerToken; 2],
}

impl<'s> JLexer<'s> {
    /// New type pattern: Generates a new lexer with given source string slice.
    pub fn new(source: &str) -> JLexer {
        JLexer{
            iter: source.char_indices(),
            last_tk: [NullToken, NullToken],
        }
    }
}

impl<'s> Iterator for JLexer<'s> {
    type Item = (JLexerToken, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // Check if last token was a StringContent.
        if let Some((i,c)) = self.iter.next() {
            let token = match c {
                whitespace_pat!() => {
                    // Check following characters, and skip the whole whitespace series.
                    while check_if_next_fits(&self.iter, is_whitespace) == Some(true) {
                        self.iter.next();
                    }
                    Whitespace
                },
                '{' => ObjectBegin,
                '}' => ObjectEnd,
                '[' => ArrayBegin,
                ']' => ArrayEnd,
                ':' => NameSeparator,
                ',' => ValueSeparator,
                '\"' => StringToken,
                't' => {
                    // Check if true-token, otherwise UnknownToken
                    TrueToken
                },
                'f' => {
                    // Check if false-token, otherwise UnknownToken
                    FalseToken
                },
                'n' => {
                    // Check if null-token, otherwise UnknownToken
                    NullToken
                },
                // StringContent(String),
                number_pat!() => {
                    // Parse number, check if integer or float.
                    let number = 0; // TODO
                    NumberInteger(number)
                    // NumberFloat(f64),
                },
                _ => UnknownToken(c.into()),
            };
            Some((token, i+1))
        } else {
            None
        }
    }
}

fn is_whitespace(c: char) -> bool {
    match c {
        whitespace_pat!() => true,
        _ => false
    }
}

fn is_number(c: char) -> bool {
    match c {
        number_pat!() => true,
        _ => false,
    }
}

/// Returns the next lexical item without modifying the original iterator.
fn crib_next(iter: &std::str::CharIndices<'_>) -> Option<(usize,char)> {
    let mut iter = iter.clone();
    iter.next()
}

fn check_if_next_is(iter: &std::str::CharIndices<'_>, c: char) -> Option<bool> {
    if let Some((_i, ci)) = crib_next(iter) {
        Some(c == ci)
    } else {
        None
    }
}

fn check_if_next_fits(iter: &std::str::CharIndices<'_>, pat: fn(char) -> bool) -> Option<bool> {
    if let Some((_i, c)) = crib_next(iter) {
        Some(pat(c))
    } else {
        None 
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_cmp {
        ($iter:expr, $value:expr, $pos:expr) => {
            assert_eq!($iter.next(), Some(($value, $pos)));
        }
    }

    #[test]
    fn varying_single_tokens() {
        let mut lexer = JLexer::new("{\n\t{\r} [],:}");
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, Whitespace, 2);
        assert_cmp!(lexer, ObjectBegin, 4);
        assert_cmp!(lexer, Whitespace, 5);
        assert_cmp!(lexer, ObjectEnd, 6);
        assert_cmp!(lexer, Whitespace, 7);
        assert_cmp!(lexer, ArrayBegin, 8);
        assert_cmp!(lexer, ArrayEnd, 9);
        assert_cmp!(lexer, ValueSeparator, 10);
        assert_cmp!(lexer, NameSeparator, 11);
        assert_cmp!(lexer, ObjectEnd, 12);
    }

    #[test]
    fn tokens_with_strings() {
        let mut lexer = JLexer::new(r#"{\n"name": "value", "other-name":"value2"}"#);
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, Whitespace, 2);
        assert_cmp!(lexer, StringToken, 3);
        assert_cmp!(lexer, StringContent(String::from("name")), 4);
        assert_cmp!(lexer, StringToken, 9);
        assert_cmp!(lexer, NameSeparator, 10);
        assert_cmp!(lexer, Whitespace, 11);
        assert_cmp!(lexer, StringToken, 12);
        assert_cmp!(lexer, StringContent(String::from("value")), 13);
        assert_cmp!(lexer, StringToken, 19);
        assert_cmp!(lexer, ValueSeparator, 20);
        assert_cmp!(lexer, Whitespace, 21);
        assert_cmp!(lexer, StringToken, 22);
        assert_cmp!(lexer, StringContent(String::from("other-name")), 23);
        assert_cmp!(lexer, StringToken, 34);
        assert_cmp!(lexer, NameSeparator, 35);
        assert_cmp!(lexer, StringToken, 36);
        assert_cmp!(lexer, StringContent(String::from("value2")), 37);
        assert_cmp!(lexer, StringToken, 44);
        assert_cmp!(lexer, ObjectEnd, 1);
    }

    #[test]
    fn tokens_with_numbers() {
        todo!();
    }
}
