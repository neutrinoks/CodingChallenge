//! Module contains the JSON-parser and the internal used JSON-Lexer.
//!
//! For more background informations on the following definitions have a look at the
//! (RFC-8259)[https://www.rfc-editor.org/rfc/rfc8259].
//!
//! ### Some notes from the RFC-8259 (Chapter 2, JSON Grammar)
//!
//! *Whitespaces* are: Space, horizontal tab, line feed or new line, CR
//!
/// Possible structural lexer-tokens:
/// - '[' as begin-array token
/// - '{' as begin-object token
/// - ']' as end-array token
/// - '}' as end-object token
/// - ':' as name-separator
/// - ',' as value-separator
use JLexerToken::*;

/// Internal macro for do-not-repeat-yourself
macro_rules! whitespace_pat {
    () => {
        ' ' | '\n' | '\r' | '\t'
    };
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

impl JLexerToken {
    pub fn is_string_content(&self) -> bool {
        matches!(self, StringContent(_))
    }
}

// type LexIterType<'s> = std::iter::Peekable<std::str::CharIndices<'s>>;
type LexIterType<'s> = std::str::CharIndices<'s>;
/// Our JSON-lexer to go through string based source.
///
///
/// Additional thoughts:
/// - Collect string-literals by '"'
/// - We do net decide whether a string is correct in terms of identifying names or regular
///   strings. We only do providing raw lexer tokens, it is the Parser's job to decide whether they
///   are correct.
/// - We have different whitespaces, so we provide only one whitespace token for multiple in a row.
/// - No token can be of zero length -> that is not a token!
///
/// From this informations we derive the possible tokens, see JLexerToken.
#[derive(Clone, Debug)]
pub struct JLexer<'s> {
    /// Reference to source text.
    source: &'s str,
    /// Internal iterator for string-based source.
    iter: LexIterType<'s>,
    /// Last tokens we identified. Last one: last_tk[1], before last one: last_tk[0].
    last_tk: [JLexerToken; 2],
}

type MidLexerOutput = Option<(JLexerToken, usize)>;

impl<'s> JLexer<'s> {
    /// New type pattern: Generates a new lexer with given source string slice.
    pub fn new(source: &str) -> JLexer {
        JLexer {
            source,
            iter: source.char_indices(),
            last_tk: [NullToken, NullToken],
        }
    }

    fn expects_string_content(&self) -> bool {
        self.last_tk[1] == StringToken
            && !(self.last_tk[0].is_string_content() || self.last_tk[0] == StringToken)
    }

    fn try_lex_string(&mut self) -> MidLexerOutput {
        seek_until(&mut self.iter, |c| c != '\"').map(|(start, stop)| {
            (
                StringContent(String::from(&self.source[start..stop])),
                start,
            )
        })
    }

    fn try_lex_number(&mut self) -> MidLexerOutput {
        seek_until(&mut self.iter, is_number).map(|(start, stop)| {
            let slice = &self.source[start..stop];
            if slice.contains('.') {
                if let Ok(number) = slice.parse::<f64>() {
                    (NumberFloat(number), start)
                } else {
                    (UnknownToken(String::from(slice)), start)
                }
            } else if let Ok(number) = slice.parse::<isize>() {
                (NumberInteger(number), start)
            } else {
                (UnknownToken(String::from(slice)), start)
            }
        })
    }

    fn try_string_token(&mut self, pat: &str, tk: JLexerToken) -> MidLexerOutput {
        seek_until(&mut self.iter, char::is_alphabetic).map(|(start, stop)| {
            let slice = &self.source[start..stop];
            if slice == pat {
                (tk, start)
            } else {
                (UnknownToken(String::from(slice)), start)
            }
        })
    }

    fn lex_structural(&mut self) -> MidLexerOutput {
        self.iter.next().map(|(p, c)| {
            let token = match c {
                whitespace_pat!() => {
                    // Check following characters, and skip the whole whitespace series.
                    seek_until(&mut self.iter, is_whitespace);
                    Whitespace
                },
                '{' => ObjectBegin,
                '}' => ObjectEnd,
                '[' => ArrayBegin,
                ']' => ArrayEnd,
                ':' => NameSeparator,
                ',' => ValueSeparator,
                '\"' => StringToken,
                _ => panic!("Return this shit to developer"),
            };
            (token, p)
        })
    }
}

impl<'s> Iterator for JLexer<'s> {
    type Item = (JLexerToken, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // First check for expected strings or possible numbers.
        if self.expects_string_content() {
            if check_if_next_is(&self.iter, '\"') {
                self.lex_structural()
            } else {
                self.try_lex_string()
            }
        } else if check_if_next_fits(&self.iter, is_number) {
            self.try_lex_number()
        } else if check_if_next_fits(&self.iter, is_structural) {
            self.lex_structural()
        } else if check_if_next_is(&self.iter, 't') {
            self.try_string_token("true", TrueToken)
        } else if check_if_next_is(&self.iter, 'f') {
            self.try_string_token("false", FalseToken)
        } else if check_if_next_is(&self.iter, 'n') {
            self.try_string_token("null", NullToken)
        } else {
            // Unknown token, extract and return it as feedback information.
            seek_until(&mut self.iter, char::is_alphabetic).map(|(start, stop)| {
                let slice = String::from(&self.source[start..stop]);
                (UnknownToken(slice), start)
            })
        }
        .map(|(tk, p)| {
            self.last_tk[0] = self.last_tk[1].clone();
            self.last_tk[1] = tk.clone();
            (tk, p + 1)
        })
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(c, whitespace_pat!())
}

fn is_number(c: char) -> bool {
    matches!(
        c,
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.'
    )
}

fn is_structural(c: char) -> bool {
    matches!(
        c,
        whitespace_pat!() | '{' | '[' | ']' | '}' | ':' | ',' | '\"'
    )
}

/// Returns the next lexical item without modifying the original iterator.
fn crib_next(iter: &LexIterType<'_>) -> Option<(usize, char)> {
    iter.clone().next()
}

// /// Checks if next char is equal to 'c' without modifying original iterator.
fn check_if_next_is(iter: &LexIterType<'_>, c: char) -> bool {
    crib_next(iter).is_some_and(|(_, ci)| ci == c)
}

/// Checks if next char matches pattern provided by function without modifying original iterator.
fn check_if_next_fits(iter: &LexIterType<'_>, pat: fn(char) -> bool) -> bool {
    crib_next(iter).is_some_and(|(_, c)| pat(c))
}

/// Methods seeks iterator forward until f_next cancels process and returns String.
/// f_next() shall return true if next does also belong to that string to be seeked, and false if
/// seeking shall stop with current character.
fn seek_until(iter: &mut LexIterType<'_>, f_next: fn(char) -> bool) -> Option<(usize, usize)> {
    let mut iter_peek = iter.clone();
    let mut start = 0;

    if let Some((p, c)) = iter_peek.next() {
        if f_next(c) {
            start = p;
            iter.next();
        }
    }
    // Be aware: this implies, that the very first character is never a sequence, because every
    // JSON-file starts with a single token ('{').
    if start == 0 {
        return None;
    }

    let mut stop = start;
    for (p, c) in iter_peek {
        if f_next(c) {
            stop = p;
            iter.next();
        } else {
            break;
        }
    }

    Some((start, stop + 1))
}

#[cfg(test)]
mod tests {
    use super::{JLexer, JLexerToken::*};

    macro_rules! assert_cmp {
        ($iter:expr, $value:expr, $pos:expr) => {
            assert_eq!($iter.next(), Some(($value, $pos)));
        };
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
        let mut lexer = JLexer::new("{\n\"name\": \"value\", \"other-name\":\"value2\"}");
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, Whitespace, 2);
        assert_cmp!(lexer, StringToken, 3);
        assert_cmp!(lexer, StringContent(String::from("name")), 4);
        assert_cmp!(lexer, StringToken, 8);
        assert_cmp!(lexer, NameSeparator, 9);
        assert_cmp!(lexer, Whitespace, 10);
        assert_cmp!(lexer, StringToken, 11);
        assert_cmp!(lexer, StringContent(String::from("value")), 12);
        assert_cmp!(lexer, StringToken, 17);
        assert_cmp!(lexer, ValueSeparator, 18);
        assert_cmp!(lexer, Whitespace, 19);
        assert_cmp!(lexer, StringToken, 20);
        assert_cmp!(lexer, StringContent(String::from("other-name")), 21);
        assert_cmp!(lexer, StringToken, 31);
        assert_cmp!(lexer, NameSeparator, 32);
        assert_cmp!(lexer, StringToken, 33);
        assert_cmp!(lexer, StringContent(String::from("value2")), 34);
        assert_cmp!(lexer, StringToken, 40);
        assert_cmp!(lexer, ObjectEnd, 41);
    }

    #[test]
    fn empty_string_literals() {
        let mut lexer = JLexer::new("\"\" \"test");
        assert_cmp!(lexer, StringToken, 1);
        assert_cmp!(lexer, StringToken, 2);
        assert_cmp!(lexer, Whitespace, 3);
        assert_cmp!(lexer, StringToken, 4);
        assert_cmp!(lexer, StringContent(String::from("test")), 5);
    }

    #[test]
    fn tokens_with_numbers() {
        let mut lexer = JLexer::new("\"age\":15,\"weight\": 55.7 ");
        assert_cmp!(lexer, StringToken, 1);
        assert_cmp!(lexer, StringContent(String::from("age")), 2);
        assert_cmp!(lexer, StringToken, 5);
        assert_cmp!(lexer, NameSeparator, 6);
        assert_cmp!(lexer, NumberInteger(15), 7);
        assert_cmp!(lexer, ValueSeparator, 9);
        assert_cmp!(lexer, StringToken, 10);
        assert_cmp!(lexer, StringContent(String::from("weight")), 11);
        assert_cmp!(lexer, StringToken, 17);
        assert_cmp!(lexer, NameSeparator, 18);
        assert_cmp!(lexer, Whitespace, 19);
        assert_cmp!(lexer, NumberFloat(55.7), 20);
        assert_cmp!(lexer, Whitespace, 24);
    }

    #[test]
    fn shortcut_floating_point_values() {
        let mut lexer = JLexer::new("{.7 10 15.}");
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, NumberFloat(0.7), 2);
        assert_cmp!(lexer, Whitespace, 4);
        assert_cmp!(lexer, NumberInteger(10), 5);
        assert_cmp!(lexer, Whitespace, 7);
        assert_cmp!(lexer, NumberFloat(15.0), 8);
        assert_cmp!(lexer, ObjectEnd, 11);
    }

    #[test]
    fn tokens_with_string_tokens() {
        let mut lexer = JLexer::new(r#"{"is_lexer": true,false null xxx false}"#);
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, StringToken, 2);
        assert_cmp!(lexer, StringContent(String::from("is_lexer")), 3);
        assert_cmp!(lexer, StringToken, 11);
        assert_cmp!(lexer, NameSeparator, 12);
        assert_cmp!(lexer, Whitespace, 13);
        assert_cmp!(lexer, TrueToken, 14);
        assert_cmp!(lexer, ValueSeparator, 18);
        assert_cmp!(lexer, FalseToken, 19);
        assert_cmp!(lexer, Whitespace, 24);
        assert_cmp!(lexer, NullToken, 25);
        assert_cmp!(lexer, Whitespace, 29);
        assert_cmp!(lexer, UnknownToken("xxx".to_string()), 30);
        assert_cmp!(lexer, Whitespace, 33);
        assert_cmp!(lexer, FalseToken, 34);
        assert_cmp!(lexer, ObjectEnd, 39);
    }

    #[test]
    fn tokens_with_empty_object_and_array() {
        let mut lexer = JLexer::new(r#"{"array": [], "object": {}}"#);
        assert_cmp!(lexer, ObjectBegin, 1);
        assert_cmp!(lexer, StringToken, 2);
        assert_cmp!(lexer, StringContent(String::from("array")), 3);
        assert_cmp!(lexer, StringToken, 8);
        assert_cmp!(lexer, NameSeparator, 9);
        assert_cmp!(lexer, Whitespace, 10);
        assert_cmp!(lexer, ArrayBegin, 11);
        assert_cmp!(lexer, ArrayEnd, 12);
        assert_cmp!(lexer, ValueSeparator, 13);
        assert_cmp!(lexer, Whitespace, 14);
        assert_cmp!(lexer, StringToken, 15);
        assert_cmp!(lexer, StringContent(String::from("object")), 16);
        assert_cmp!(lexer, StringToken, 22);
        assert_cmp!(lexer, NameSeparator, 23);
        assert_cmp!(lexer, Whitespace, 24);
        assert_cmp!(lexer, ObjectBegin, 25);
        assert_cmp!(lexer, ObjectEnd, 26);
        assert_cmp!(lexer, ObjectEnd, 27);
    }
}
