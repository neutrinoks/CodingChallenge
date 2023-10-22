//! The JSON-Parser implementation. It uses the JLexer for a first lexing of the given source.
//!
//! For more background informations on the following definitions have a look at the 
//! (RFC-8259)[https://www.rfc-editor.org/rfc/rfc8259].
//!
//! ### Some notes from the RFC-8259 (Chapter 3-7, Values, Objects, Arrays, Numbers)
//!
//! *Values* are: Object, array, number, string, or true/false/null.
//!
//! *Objects* are: begin-object [ member (value-separator member)* ] end-object, where
//! member is: string name-separator value.
//!
//! *Arrays* are: begin-array [ value (value-separator value)* end-array
//!
//! *Numbers* are: [minus] int [frag][exp]; and can contain decimal-point '.', digits1-9 '1'-'9',
//! e 'e'|'E', exp: e [minus|plus] 1*DIGIT, frag: decimal-point 1*DIGIT etc.
//!
//! *Strings* are: quotation-mark char* quotation-mark; where char: escaped | unescaped, TODO!

use crate::jlexer::{JLexer, JLexerToken as JLToken};


#[derive(Debug, PartialEq)]
enum JPartialValue {
    Float(f64),
    Integer(isize),
    String(String),
    True,
    False,
    Null,
}

#[derive(Debug, PartialEq)]
enum JPartialToken {
    ObjectBegin,
    ObjectEnd,
    Array(Vec<JPartialValue>),
    MemberName(String),
    MemberValue(JPartialValue),
}

/// Possible errors thrown by the JParser.
#[derive(Debug, Clone, PartialEq)]
pub enum JParserError {
    /// If source contains no main object: '{ }'.
    NoBeginningObject,
    /// If some object is not closed properly, missing '}'.
    UnclosedObject,
    /// Unexpected token at this position.
    UnexpectedToken(String),
    /// Unknown token was returned from the lexer.
    UnknownToken(String),
}

impl std::fmt::Display for JParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for JParserError {}

/// A generic Result for JParser.
pub type JPResult<T> = Result<T, JParserError>;

/// An internal and parser-related identifier for begin-object and begin-array.
#[derive(Debug, PartialEq)]
enum StackIdent {
    BeginObject,
    BeginArray,
}

/// This is a reduced variant of JPartialToken, only interpreted as expection of JPartialParser.
#[derive(Debug, PartialEq)]
enum JPartialExpect {
    ObjectBegin,
    ObjectEnd,
    NameSeparator,
    ValueSeparator,
    MemberName,
    MemberValue,
    Unknown,
}

impl PartialEq<JPartialToken> for JPartialExpect {
    fn eq(&self, other: &JPartialToken) -> bool {
        false
    }
}

/// The JParser on top of the JLexer checks for a proper syntax/structure of the JSON-file.
///
/// Because every JSON file needs at least contain '{}', which is some kind of main-object, this
/// will not be an output! If it is not contained, an error will be thrown! But the first element
/// would be the first output.
struct JPartialParser<'s> {
    /// Internal lexer to go through source token by token.
    lexer: std::iter::Filter<JLexer<'s>, fn(&(JLToken,usize)) -> bool>,
    /// Stack for begin-object, begin-array.
    stack: Vec<StackIdent>,
    /// Buffer of last two parsed tokens.
    last_tk: [JPartialExpect; 2],
    /// Expectation for next token, dependent on JSON grammar.
    expect: Vec<JPartialExpect>,
    /// Counter of parsed elements.
    count: usize,
}

impl<'s> JPartialParser<'s> {
    /// New type pattern, to create a new JParser for a given source.
    pub fn new(source: &'s str) -> JPartialParser<'s> {
        JPartialParser{
            lexer: JLexer::new(source).filter(|(ltk,_)| !matches!(ltk, JLToken::Whitespace)),
            stack: Vec::new(),
            last_tk: [JPartialExpect::Unknown, JPartialExpect::Unknown],
            expect: vec![JPartialExpect::ObjectBegin],
            count: 0,
        }
    }

    fn was_expected(&self, ltk: &JLToken, p: usize) -> JPResult<()> {
        // Try to find an expected one.
        for tk in &self.expect {
            let is_ok = match tk {
                JPartialExpect::ObjectBegin => matches!(ltk, JLToken::ObjectBegin),
                JPartialExpect::ObjectEnd => matches!(ltk, JLToken::ObjectEnd),
                JPartialExpect::NameSeparator => matches!(ltk, JLToken::NameSeparator),
                JPartialExpect::ValueSeparator => matches!(ltk, JLToken::ValueSeparator),
                JPartialExpect::MemberName => matches!(ltk, JLToken::StringContent(_)),
                JPartialExpect::MemberValue => matches!(ltk, 
                    JLToken::ArrayBegin | JLToken::ObjectBegin | JLToken::StringContent(_) | 
                    JLToken::NumberFloat(_) | JLToken::NumberInteger(_) | JLToken::NullToken |
                    JLToken::TrueToken | JLToken::FalseToken),
                JPartialExpect::Unknown => true,
            };
            if is_ok {
                return Ok(())
            }
        }

        // Otherwise build error message and return error.
        let mut e_iter = self.expect.iter();
        let mut errmsg = format!("at position {} expected: {:?}", p, e_iter.next().unwrap());
        for exp in e_iter {
            errmsg.push_str(&format!(" or {:?}", exp));
        }
        errmsg.push_str(&format!("\nfound: {:?}", ltk));
        Err(JParserError::UnexpectedToken(errmsg))
    }
}

impl<'s> Iterator for JPartialParser<'s> {
    type Item = JPResult<(JPartialToken, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|(ltk,p)| {
            // Check for first grammar errors (if was expected).
            self.was_expected(&ltk, p)?;

            match ltk {
                JLToken::ObjectBegin => {
                    // Setup expectations
                    Ok((JPartialToken::ObjectBegin, p))
                },
                JLToken::ArrayBegin => {
                    // Setup expectations
                    // Parse array
                    Ok((JPartialToken::Array(Vec::new()), p))
                },
                JLToken::TrueToken => {
                    // Setup expectations
                    Ok((JPartialToken::MemberValue(JPartialValue::True), p))
                },
                JLToken::FalseToken => {
                    // Setup expectations
                    Ok((JPartialToken::MemberValue(JPartialValue::False), p))
                },
                JLToken::NullToken => {
                    // Setup expectations
                    Ok((JPartialToken::MemberValue(JPartialValue::Null), p))
                },
                JLToken::StringContent(s) => {
                    // Check tokens before, either MemberName or MemberValue...
                    // Setup expectations
                    // Ok((JPartialToken::token, p))
                    todo!()
                },
                JLToken::NumberInteger(i) => {
                    // Setup expectations
                    Ok((JPartialToken::MemberValue(JPartialValue::Integer(i)), p))
                },
                JLToken::NumberFloat(f) => {
                    // Setup expectations
                    Ok((JPartialToken::MemberValue(JPartialValue::Float(f)), p))
                },
                JLToken::UnknownToken(s) => {
                    let errmsg = format!("Unknown token \"{}\" at position {}", s, p);
                    Err(JParserError::UnknownToken(errmsg))
                },
                _ => {
                    // Should not appear due to the concept of algorithm:
                    // JLToken::Whitespace, JLToken::ObjectEnd, JLToken::ArrayEnd,
                    // JLToken::NameSeparator, JLToken::ValueSeparator, JLToken::StringToken
                    panic!("Whitespaces should all be filtered out!")
                },
            }
        })
    }
}


pub struct JParser {
    partial_tokens: Vec<(JPartialToken, usize)>,
    // pub tokens ...
}

impl JParser {
    /// New type pattern, creates a new parser from given source.
    pub fn new<'s>(source: &'s str) -> JPResult<JParser> {
        let mut jpart_parser = JPartialParser::new(source);
        let mut partial_tokens: Vec<(JPartialToken, usize)> = Vec::new();
        while let Some(tk) = jpart_parser.next() {
            let tk = tk?;
            partial_tokens.push(tk);
        }

        Ok(JParser{ partial_tokens })
    }

}


/// Runs a full lexical analysis of the source.
pub fn json_full_analysis(source: &str) -> JPResult<()> {
    Err(JParserError::NoBeginningObject)
}


#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_cmp {
        ($iter:expr, $value:expr, $pos:expr) => {
            assert_eq!($iter.next(), Some(Ok(($value, $pos))));
        };
    }

    #[test]
    fn partial_parse_main_object_one_value() {
        let mut parser = JPartialParser::new("{\n\"name\": 50.7\n}");
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("name".into()), 3);
        assert_cmp!(parser, JPartialToken::MemberValue(JPartialValue::Float(50.7)), 11);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 16);
    }
}
