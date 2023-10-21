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


/// This trait defines some common functions for the parsing process of each possible JParserToken.
trait ParsableToken {
    // Some common creation method to create new ones.

    // Probably some common parsing technique to unify the process of parsing.
}


#[derive(Debug, Clone)]
pub enum JNumber {
    Integer(isize),
    Float(f64),
}

/// Value type of the JSON syntax.
#[derive(Debug, Clone)]
pub enum JValue {
    Object,
    Array,
    Number(JNumber),
    String(String),
    True,
    False,
    Null,
}

impl From<JNumber> for JValue {
    fn from(num: JNumber) -> JValue {
        JValue::Number(num)
    }
}

#[derive(Debug, Clone)]
pub struct JMember {
    name: String,
    value: JValue,
}

impl JMember {
    pub(crate) fn from_keypair<P>(name: &str, value: P) -> JMember 
    where P: Into<JValue>
    {
        JMember{ name: name.to_string(), value: value.into() }
    }
}

#[derive(Debug, Clone)]
pub struct JObject {
    /// members
    pub member: Vec<JValue>,
}

/// Possible JParserToken from a syntactical perspective of the output of the JLexer.
#[derive(Debug, Clone)]
pub enum JParserToken {
    Object(JObject),
    Value(JValue),
    Array,
    Number,
    String,
}


/// Possible errors thrown by the JParser.
#[derive(Debug, Clone)]
pub enum JParserError {
    /// If source contains no main object: '{ }'.
    NoMainObject,
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
#[derive(Debug)]
enum StackIdent {
    BeginObject,
    BeginArray,
}


/// The JParser on top of the JLexer checks for a proper syntax/structure of the JSON-file.
///
/// Because every JSON file needs at least contain '{}', which is some kind of main-object, this
/// will not be an output! If it is not contained, an error will be thrown! But the first element
/// would be the first output.
pub struct JParser<'s> {
    /// Internal lexer to go through source token by token.
    lexer: JLexer<'s>,
    /// Stack for begin-object, begin-array.
    stack: Vec<StackIdent>,
    /// Counter of parsed elements.
    count: usize,
}

impl<'s> JParser<'s> {
    /// New type pattern, to create a new JParser for a given source.
    pub fn new(source: &'s str) -> JParser<'s> {
        JParser{
            lexer: JLexer::new(source),
            stack: Vec::new(),
            count: 0,
        }
    }

    /// Runs a full lexical analysis of the source.
    pub fn full_analysis(&self) -> JPResult<()> {
        Err(JParserError::NoMainObject)
    }
}

impl<'s> Iterator for JParser<'s> {
    type Item = (JParserToken, usize);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}


#[cfg(test)]
mod tests {
    use crate::assert_cmp;
    use super::{JMember, JObject, JParser, JValue, JNumber};

    #[test]
    fn parse_main_object_one_value() {
        let mut parser = JParser::new("{\n\"name\": 50.7\n}");
        let value = JMember::from_keypair("name", JNumber::Float(50.7));
        assert_cmp!(parser, value, 3);
    }
}
