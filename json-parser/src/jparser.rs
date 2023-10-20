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

impl JLexerToken {
    pub fn is_string_content(&self) -> bool {
        match self {
            StringContent(_) => true,
            _ => false,
        }
    }

    pub fn is_number_integer(&self) -> bool {
        match self {
            NumberInteger(_) => true,
            _ => false,
        }
    }

    pub fn is_number_float(&self) -> bool {
        match self {
            NumberFloat(_) => true,
            _ => false,
        }
    }

    pub fn is_unknown_token(&self) -> bool {
        match self {
            UnknownToken(_) => true,
            _ => false,
        }
    }
}

// type LexIterType<'s> = std::iter::Peekable<std::str::CharIndices<'s>>;
type LexIterType<'s> = std::str::CharIndices<'s>;
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
/// - No token can be of zero length -> that is not a token!
/// 
/// From this informations we derive the possible tokens, see JLexerToken.
pub struct JLexer<'s> {
    /// Reference to source text.
    source: &'s str,
    /// Internal iterator for string-based source.
    // iter: std::str::CharIndices<'s>,
    iter: LexIterType<'s>,
    /// Last tokens we identified. Last one: last_tk[1], before last one: last_tk[0].
    last_tk: [JLexerToken; 2],
}

type MidLexerOutput = Option<(JLexerToken, usize)>;

impl<'s> JLexer<'s> {
    /// New type pattern: Generates a new lexer with given source string slice.
    pub fn new(source: &str) -> JLexer {
        JLexer{
            source,
            iter: source.char_indices(),
            last_tk: [NullToken, NullToken],
        }
    }

    fn expects_string_content(&self) -> bool {
        self.last_tk[1] == StringToken && 
            !(self.last_tk[0].is_string_content() || self.last_tk[0] == StringToken)
    }

    fn try_lex_string(&mut self) -> MidLexerOutput {
        seek_until(&mut self.iter, |c| c != '\"')
            .and_then(|(start,stop)| {
                Some((StringContent(String::from(&self.source[start..stop])), start))
            })
    }

    fn lex_non_string(&mut self) -> MidLexerOutput {
        if let Some((p,c)) = self.iter.next() {
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
            Some((token,p))
        } else {
            None
        }
    }

    // fn finished_string_lexing(&self) -> bool {
    //     self.last[0].is_string_content() && self.last_tk[1] == StringToken
    // }
}

impl<'s> Iterator for JLexer<'s> {
    type Item = (JLexerToken, usize);

    fn next(&mut self) -> Option<Self::Item> {
        // This whole if-else-block shall return Option<(JLexerToken,usize)>
        if self.expects_string_content() {
            let result = self.try_lex_string();
            if result.is_none() {
                self.lex_non_string()
            } else {
                result
            }
        } else {
            self.lex_non_string()
        }
        .and_then(|(tk,p)| {
            self.last_tk[0] = self.last_tk[1].clone();
            self.last_tk[1] = tk.clone();
            println!("found LexerToken: {:?} at pos {}", tk, p);
            Some((tk, p+1))
        })
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
fn crib_next(iter: &LexIterType<'_>) -> Option<(usize,char)> {
    iter.clone().next()
}

/// Checks if next char is equal to 'c' without modifying original iterator.
fn check_if_next_is(iter: &LexIterType<'_>, c: char) -> Option<bool> {
    crib_next(iter).map(|(_i,ci)| ci == c)
}

/// Checks if next char matches pattern provided by function without modifying original iterator.
fn check_if_next_fits(iter: &LexIterType<'_>, pat: fn(char) -> bool) -> Option<bool> {
    crib_next(iter).map(|(_i,c)| pat(c))
}

/// Methods seeks iterator forward until f_next cancels process and returns String.
/// f_next() shall return true if next does also belong to that string to be seeked, and false if
/// seeking shall stop with current character.
fn seek_until(iter: &mut LexIterType<'_>, f_next: fn(char) -> bool) -> Option<(usize, usize)> {
    // println!("start seeking...");
    let mut iter_peek = iter.clone();
    let mut start = 0;

    if let Some((p,c)) = iter_peek.next()  {
        if f_next(c) {
            // println!("first character is {} at pos {}", c, p);
            start = p;
            iter.next();
        }
    }
    if start == 0 { return None }

    let mut stop = start;
    while let Some((p, c)) = iter_peek.next() {
        if f_next(c) {
            stop = p;
            iter.next();
            // println!("current is {} and we continue", c);
        } else {
            // println!("current is {} and we stop seeking", c);
            break
        }
    }
    
    // println!("we took string slice [{}..{}]", start, stop+1);
    Some((start, stop+1))
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

        let mut lexer = JLexer::new("\"\" \"test");
        assert_cmp!(lexer, StringToken, 1);
        assert_cmp!(lexer, StringToken, 2);
        assert_cmp!(lexer, Whitespace, 3);
        assert_cmp!(lexer, StringToken, 4);
        assert_cmp!(lexer, StringContent(String::from("test")), 5);
    }

    #[test]
    fn tokens_with_numbers() {
        todo!();
        let mut lexer = JLexer::new("\"age\": 15, \"weight\": 55.7");
    }
}
