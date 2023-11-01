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

use crate::{
    jlexer::{JLexer, JLexerToken as JLToken},
    jparser_types::{JMember, JObject, JPartialValue as JPValue, JValue},
};

const PANICSTR: &str = "Return this shit to developer!";

#[macro_export]
macro_rules! unexpected_token {
    ($pos:expr, $found:expr, $expect:expr) => {
        Err(JParseError::UnexpectedToken(
            $pos,
            UnexpTokenFeedb::from($found),
            UnexpTokenFeedb::from($expect),
        ))
    };
}

/// Possible tokens created by the JPartialParser as input for JParser.
#[derive(Clone, Debug, PartialEq)]
pub enum JPartialToken {
    ObjectBegin,
    ObjectEnd,
    Array(Vec<JPValue>),
    MemberName(String),
    MemberValue(JPValue),
}

/// This is a reduced variant of JPartialToken, only interpreted as expection of JPartialParser.
/// The purpose of this definition is controling the syntax and grammar of the json-source.
#[derive(Clone, Debug, PartialEq)]
pub enum JPartialExpect {
    ObjectBegin,
    ObjectEnd,
    MemberName,
    MemberValue,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnexpTokenFeedb {
    JPExpect(Vec<JPartialExpect>),
    JPToken(JPartialToken),
    JLToken(JLToken),
}

impl From<&Vec<JPartialExpect>> for UnexpTokenFeedb {
    fn from(vpe: &Vec<JPartialExpect>) -> UnexpTokenFeedb {
        UnexpTokenFeedb::JPExpect(vpe.clone())
    }
}

impl From<&JPartialToken> for UnexpTokenFeedb {
    fn from(jtk: &JPartialToken) -> UnexpTokenFeedb {
        UnexpTokenFeedb::JPToken(jtk.clone())
    }
}

impl From<&JLToken> for UnexpTokenFeedb {
    fn from(ltk: &JLToken) -> UnexpTokenFeedb {
        UnexpTokenFeedb::JLToken(ltk.clone())
    }
}

impl From<JPartialToken> for UnexpTokenFeedb {
    fn from(jtk: JPartialToken) -> UnexpTokenFeedb {
        UnexpTokenFeedb::JPToken(jtk)
    }
}

impl From<JLToken> for UnexpTokenFeedb {
    fn from(ltk: JLToken) -> UnexpTokenFeedb {
        UnexpTokenFeedb::JLToken(ltk)
    }
}

/// Possible errors thrown by the JParser regarding grammar or token errors of the json-source.
#[derive(Clone, Debug, PartialEq)]
pub enum JParseError {
    /// If source contains no main object: '{ }'.
    NoBeginningObject(usize),
    /// If some object is not closed properly, missing '}'.
    UnclosedObject(usize),
    /// If an array was not closed by ']'.
    UnclosedArray(usize),
    /// Unexpected end of source.
    UnexpectedEnd(usize),
    /// Unexpected token at this position, what we found and what we expected.
    UnexpectedToken(usize, UnexpTokenFeedb, UnexpTokenFeedb),
    /// Unknown token was returned from the lexer.
    UnknownToken(usize, String),
}

impl std::fmt::Display for JParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for JParseError {}

/// A generic Result for JParser.
pub type JPResult<T> = Result<T, JParseError>;

/// Internal iterator type of JPartialParser.
type JPartialParseIter<'s> = std::iter::Filter<JLexer<'s>, fn(&(JLToken, usize)) -> bool>;

/// The JParser on top of the JLexer checks for a proper syntax/structure of the JSON-file.
///
/// Because every JSON file needs at least contain '{}', which is some kind of main-object, this
/// will not be an output! If it is not contained, an error will be thrown! But the first element
/// would be the first output.
struct JPartialParser<'s> {
    /// Internal lexer to go through source token by token.
    lexer: JPartialParseIter<'s>,
    /// Expectation for next token, dependent on JSON grammar.
    expect: Vec<JPartialExpect>,
    /// Stack for begin-object, begin-array.
    object_cnt: usize,
    /// Counter of parsed elements.
    count: usize,
}

impl<'s> JPartialParser<'s> {
    /// New type pattern, to create a new JParser for a given source.
    pub fn new(source: &'s str) -> JPartialParser<'s> {
        JPartialParser {
            lexer: JLexer::new(source)
                .filter(|(ltk, _)| !matches!(ltk, JLToken::Whitespace | JLToken::StringToken)),
            expect: vec![JPartialExpect::ObjectBegin],
            object_cnt: 0,
            count: 0,
        }
    }

    fn was_expected(&self, ltk: &JLToken, p: usize) -> JPResult<()> {
        // Try to find an expected one.
        for tk in &self.expect {
            let is_ok = match tk {
                JPartialExpect::ObjectBegin => matches!(ltk, JLToken::ObjectBegin),
                JPartialExpect::ObjectEnd => matches!(ltk, JLToken::ObjectEnd),
                JPartialExpect::MemberName => matches!(ltk, JLToken::StringContent(_)),
                JPartialExpect::MemberValue => matches!(
                    ltk,
                    JLToken::ArrayBegin
                        | JLToken::ObjectBegin
                        | JLToken::StringContent(_)
                        | JLToken::NumberFloat(_)
                        | JLToken::NumberInteger(_)
                        | JLToken::NullToken
                        | JLToken::TrueToken
                        | JLToken::FalseToken
                ),
            };
            if is_ok {
                return Ok(());
            }
        }

        unexpected_token!(p, ltk, &self.expect)
    }

    fn do_we_expect(&self, exp: JPartialExpect) -> bool {
        for e in &self.expect {
            if *e == exp {
                return true;
            }
        }
        false
    }

    fn next_shall_be(&mut self, exp: JLToken, p: usize) -> JPResult<()> {
        let next = self.lexer.next();
        if next.is_none() {
            return Err(JParseError::UnexpectedEnd(p));
        }

        let next = next.unwrap();
        if exp == next.0 {
            Ok(())
        } else {
            unexpected_token!(p, &next.0, &exp)
        }
    }

    fn crib_if_next_is(&self, jlt: JLToken) -> bool {
        if let Some((tk, _)) = self.lexer.clone().next() {
            tk == jlt
        } else {
            false
        }
    }

    fn set_expect_after_member_value(&mut self) {
        if self.crib_if_next_is(JLToken::ValueSeparator) {
            self.lexer.next();
            self.expect = vec![JPartialExpect::MemberName];
        } else {
            self.expect = vec![JPartialExpect::ObjectEnd];
        }
    }
}

impl<'s> Iterator for JPartialParser<'s> {
    type Item = JPResult<(JPartialToken, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer.next().map(|(ltk, p)| {
            // Check for first grammar errors (if was expected).
            self.was_expected(&ltk, p)?;

            let tk_res: JPResult<(JPartialToken, usize)> = match ltk {
                JLToken::ObjectBegin => {
                    self.expect = vec![JPartialExpect::MemberName, JPartialExpect::ObjectEnd];
                    self.object_cnt += 1;
                    Ok((JPartialToken::ObjectBegin, p))
                }
                JLToken::ObjectEnd => {
                    if self.object_cnt > 0 {
                        self.object_cnt -= 1;
                        self.set_expect_after_member_value();
                        Ok((JPartialToken::ObjectEnd, p))
                    } else {
                        Err(JParseError::UnclosedObject(p))
                    }
                }
                JLToken::ArrayBegin => {
                    let mut array: Vec<JPValue> = Vec::new();
                    if !self.crib_if_next_is(JLToken::ArrayEnd) {
                        while let Some((ltk, pi)) = self.lexer.next() {
                            match ltk {
                                JLToken::StringContent(s) => array.push(JPValue::String(s)),
                                JLToken::NumberInteger(i) => array.push(JPValue::Integer(i)),
                                JLToken::NumberFloat(f) => array.push(JPValue::Float(f)),
                                JLToken::TrueToken => array.push(JPValue::True),
                                JLToken::FalseToken => array.push(JPValue::False),
                                JLToken::NullToken => array.push(JPValue::Null),
                                _ => return unexpected_token!(pi, ltk, &self.expect),
                            }
                            if self.crib_if_next_is(JLToken::ValueSeparator) {
                                self.next();
                            } else {
                                break;
                            }
                        }
                    }
                    self.next_shall_be(JLToken::ArrayEnd, p)?;
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::Array(array), p))
                }
                JLToken::TrueToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::True), p))
                }
                JLToken::FalseToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::False), p))
                }
                JLToken::NullToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Null), p))
                }
                JLToken::StringContent(s) => {
                    if self.do_we_expect(JPartialExpect::MemberName) {
                        self.next_shall_be(JLToken::NameSeparator, p)?;
                        self.expect =
                            vec![JPartialExpect::MemberValue, JPartialExpect::ObjectBegin];
                        Ok((JPartialToken::MemberName(s), p))
                    } else if self.do_we_expect(JPartialExpect::MemberValue) {
                        self.set_expect_after_member_value();
                        Ok((JPartialToken::MemberValue(JPValue::String(s)), p))
                    } else {
                        panic!("{}", PANICSTR)
                    }
                }
                JLToken::NumberInteger(i) => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Integer(i)), p))
                }
                JLToken::NumberFloat(f) => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Float(f)), p))
                }
                JLToken::UnknownToken(s) => Err(JParseError::UnknownToken(p, s)),
                _ => {
                    // Should not appear due to the concept of algorithm:
                    // JLToken::Whitespace, JLToken::ArrayEnd, JLToken::NameSeparator,
                    // JLToken::ValueSeparator, JLToken::StringToken
                    panic!("{}", PANICSTR)
                }
            };
            self.count += 1;
            tk_res
        })
    }
}

/// The JParser
pub struct JParser<'s>(JPartialParser<'s>);

impl<'s> JParser<'s> {
    /// New type pattern.
    pub fn new(source: &'s str) -> JParser<'s> {
        JParser(JPartialParser::new(source))
    }

    /// New type pattern, creates a new parser from given source.
    pub fn parse(&mut self) -> JPResult<JObject> {
        // Consume first object-begin and parse the main object...
        if let Some(_result) = self.0.next() {
            self.parse_object()
        } else {
            Err(JParseError::NoBeginningObject(1))
        }
    }

    /// Method starts with inner content, the object-begin was already consumed.
    fn parse_object(&mut self) -> JPResult<JObject> {
        let mut object = JObject::default();
        loop {
            // At this point, there should be only member-name or object-end!
            let jtk = self.0.next().unwrap()?.0;
            let name = match jtk {
                JPartialToken::MemberName(name) => name,
                JPartialToken::ObjectEnd => break,
                _ => panic!("{}", PANICSTR),
            };

            // Here, we only expect member-values (single values, arrays and objects).
            let jtk = self.0.next().unwrap()?.0;
            let value = match jtk {
                JPartialToken::MemberValue(val) => JValue::from(val),
                JPartialToken::Array(array) => JValue::Array(array),
                JPartialToken::ObjectBegin => {
                    let result = self.parse_object();
                    JValue::Object(result?)
                }
                _ => panic!("{}", PANICSTR),
            };

            object.members.push(JMember { name, value });
        }
        Ok(object)
    }
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
    fn partial_parse_main_empty_and_one_value() {
        let mut parser = JPartialParser::new("{}");
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 2);

        let mut parser = JPartialParser::new("{\n\"name\": 50.7\n}");
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("name".into()), 4);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::Float(50.7)), 11);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 16);
    }

    #[test]
    fn partial_parse_main_multiple_members() {
        let mut parser = JPartialParser::new(
            r#"{"name": "Michael", "has_job": true, "has_kid": false, "pointer": null, "features": ["test", 10, true]}"#,
        );
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("name".to_string()), 3);
        assert_cmp!(
            parser,
            JPartialToken::MemberValue(JPValue::from("Michael")),
            11
        );
        assert_cmp!(parser, JPartialToken::MemberName("has_job".to_string()), 22);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::True), 32);
        assert_cmp!(parser, JPartialToken::MemberName("has_kid".to_string()), 39);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::False), 49);
        assert_cmp!(parser, JPartialToken::MemberName("pointer".to_string()), 57);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::Null), 67);
        assert_cmp!(
            parser,
            JPartialToken::MemberName("features".to_string()),
            74
        );
        let array = vec![
            JPValue::String("test".to_string()),
            JPValue::Integer(10),
            JPValue::True,
        ];
        assert_cmp!(parser, JPartialToken::Array(array), 85);
    }

    #[test]
    fn partial_parse_arrays() {
        let mut parser = JPartialParser::new(
            r#"{
            "key1": ["test", true, false],
            "key2": [],
            "key3": [null, 15, 7.5]
            }"#,
        );
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("key1".to_string()), 16);
        let array = vec![
            JPValue::String("test".to_string()),
            JPValue::True,
            JPValue::False,
        ];
        assert_cmp!(parser, JPartialToken::Array(array), 23);
        assert_cmp!(parser, JPartialToken::MemberName("key2".to_string()), 59);
        assert_cmp!(parser, JPartialToken::Array(Vec::new()), 66);
        assert_cmp!(parser, JPartialToken::MemberName("key3".to_string()), 83);
        let array = vec![JPValue::Null, JPValue::Integer(15), JPValue::Float(7.5)];
        assert_cmp!(parser, JPartialToken::Array(array), 90);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 118);
    }

    #[test]
    fn partial_parse_objects() {
        let mut parser = JPartialParser::new(
            r#"{
            "object": { 
                "data": "data", 
                "object2": {}
            } 
        }"#,
        );
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("object".to_string()), 16);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 25);
        assert_cmp!(parser, JPartialToken::MemberName("data".to_string()), 45);
        assert_cmp!(
            parser,
            JPartialToken::MemberValue(JPValue::from("data")),
            53
        );
        assert_cmp!(parser, JPartialToken::MemberName("object2".to_string()), 78);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 88);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 89);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 103);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 114);
    }
}
