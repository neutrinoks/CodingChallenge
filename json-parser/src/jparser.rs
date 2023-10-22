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

use crate::jlexer::{JLexer, JLexerToken::*};


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
    Unknown, // lalalalala
}

/// Possible errors thrown by the JParser.
#[derive(Debug, Clone, PartialEq)]
pub enum JParserError {
    /// If source contains no main object: '{ }'.
    NoBeginningObject,
    /// If some object is not closed properly, missing '}'.
    UnclosedObject,
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


/// The JParser on top of the JLexer checks for a proper syntax/structure of the JSON-file.
///
/// Because every JSON file needs at least contain '{}', which is some kind of main-object, this
/// will not be an output! If it is not contained, an error will be thrown! But the first element
/// would be the first output.
struct JPartialParser<'s> {
    /// Internal lexer to go through source token by token.
    lexer: JLexer<'s>,
    /// Stack for begin-object, begin-array.
    stack: Vec<StackIdent>,
    /// Buffer of last two parsed tokens.
    last_tk: [JPartialToken; 2],
    /// Counter of parsed elements.
    count: usize,
}

impl<'s> JPartialParser<'s> {
    /// New type pattern, to create a new JParser for a given source.
    pub fn new(source: &'s str) -> JPartialParser<'s> {
        JPartialParser{
            lexer: JLexer::new(source),
            stack: Vec::new(),
            last_tk: [JPartialToken::Unknown, JPartialToken::Unknown],
            count: 0,
        }
    }

}

impl<'s> Iterator for JPartialParser<'s> {
    type Item = JPResult<(JPartialToken, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|(ltk,p)| {
            Err(JParserError::NoBeginningObject)
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
