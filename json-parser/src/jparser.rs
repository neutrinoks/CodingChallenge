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
    jparser_types::{JPartialValue as JPValue, JValue, JObject, JMember},
};


macro_rules! unexpected_token {
    ($exp:expr, $pos:expr) => {
        {
            let errmsg = format!("expected {:?} at position {}", $exp, $pos);
            Err(JParserError::UnexpectedToken(errmsg))
        }
    };
    ($exp:expr, $pos:expr, $inst:expr) => {
        {
            let msg = format!("expected {:?} at position {} - found {:?}", $exp, $pos, $inst);
            Err(JParserError::UnexpectedToken(msg))
        }
    };
}

/// Possible tokens created by the JPartialParser as input for JParser.
#[derive(Debug, PartialEq)]
enum JPartialToken {
    ObjectBegin,
    ObjectEnd,
    Array(Vec<JPValue>),
    MemberName(String),
    MemberValue(JPValue),
}

/// This is a reduced variant of JPartialToken, only interpreted as expection of JPartialParser.
/// The purpose of this definition is controling the syntax and grammar of the json-source.
#[derive(Debug, PartialEq)]
enum JPartialExpect {
    ObjectBegin,
    ObjectEnd,
    MemberName,
    MemberValue,
}

/// Possible errors thrown by the JParser regarding grammar or token errors of the json-source.
#[derive(Debug, Clone, PartialEq)]
pub enum JParserError {
    /// If source contains no main object: '{ }'.
    NoBeginningObject,
    /// If some object is not closed properly, missing '}'.
    UnclosedObject,
    /// If an array was not closed by ']'.
    UnclosedArray,
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

/// Internal iterator type of JPartialParser.
type JPartialParseIter<'s> = std::iter::Filter<JLexer<'s>, fn(&(JLToken,usize)) -> bool>;

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
        JPartialParser{
            lexer: JLexer::new(source).filter(|(ltk,_)| 
                !matches!(ltk, JLToken::Whitespace | JLToken::StringToken)),
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
                JPartialExpect::MemberValue => matches!(ltk, 
                    JLToken::ArrayBegin | JLToken::ObjectBegin | JLToken::StringContent(_) | 
                    JLToken::NumberFloat(_) | JLToken::NumberInteger(_) | JLToken::NullToken |
                    JLToken::TrueToken | JLToken::FalseToken),
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
        errmsg.push_str(&format!(" - found: {:?}", ltk));
        Err(JParserError::UnexpectedToken(errmsg))
    }

    fn do_we_expect(&self, exp: JPartialExpect) -> bool {
        for e in &self.expect {
            if *e == exp {
                return true
            }
        }
        false
    }

    fn next_shall_be(&mut self, exp: JLToken, p: usize) -> JPResult<()> {
        let next = self.lexer.next();
        if next.is_none() {
            return unexpected_token!(exp, p)
        }

        let next = next.unwrap();
        if exp == next.0 {
            Ok(())
        } else {
            unexpected_token!(exp, p, next)
        }
    }

    fn crib_if_next_is(&self, jlt: JLToken) -> bool {
        if let Some((tk,_)) = self.lexer.clone().next() {
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
        self.lexer.next().map(|(ltk,p)| {
            // Check for first grammar errors (if was expected).
            self.was_expected(&ltk, p)?;

            let token = match ltk {
                JLToken::ObjectBegin => {
                    self.expect = vec![JPartialExpect::MemberName, JPartialExpect::ObjectEnd];
                    self.object_cnt += 1;
                    Ok((JPartialToken::ObjectBegin, p))
                },
                JLToken::ObjectEnd => {
                    if self.object_cnt > 0 {
                        self.object_cnt -= 1;
                        Ok((JPartialToken::ObjectEnd, p))
                    } else {
                        Err(JParserError::UnclosedObject)
                    }
                },
                JLToken::ArrayBegin => {
                    let mut array: Vec<JPValue> = Vec::new();
                    let mut p = p; // In this case we need to modify it.
                    while let Some((ltk,pi)) = self.lexer.next() {
                        match ltk {
                            JLToken::StringContent(s) => array.push(JPValue::String(s)),
                            JLToken::NumberInteger(i) => array.push(JPValue::Integer(i)),
                            JLToken::NumberFloat(f) => array.push(JPValue::Float(f)),
                            JLToken::TrueToken => array.push(JPValue::True),
                            JLToken::FalseToken => array.push(JPValue::True),
                            JLToken::NullToken => array.push(JPValue::Null),
                            _ => return unexpected_token!("JPValue", pi, ltk),
                        }
                        if self.crib_if_next_is(JLToken::ValueSeparator) {
                            self.next();
                        } else {
                            p = pi;
                            break
                        }
                    }
                    self.next_shall_be(JLToken::ArrayEnd, p)?;
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::Array(array), p))
                },
                JLToken::TrueToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::True), p))
                },
                JLToken::FalseToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::False), p))
                },
                JLToken::NullToken => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Null), p))
                },
                JLToken::StringContent(s) => {
                    if self.do_we_expect(JPartialExpect::MemberName) {
                        self.next_shall_be(JLToken::NameSeparator, p)?;
                        self.expect = vec![
                            JPartialExpect::MemberValue, JPartialExpect::ObjectBegin
                        ];
                        Ok((JPartialToken::MemberName(s), p))
                    } else if self.do_we_expect(JPartialExpect::MemberValue) {
                        self.set_expect_after_member_value();
                        Ok((JPartialToken::MemberValue(JPValue::String(s)), p))
                    } else {
                        panic!("Return this shit to developer!")
                    }
                },
                JLToken::NumberInteger(i) => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Integer(i)), p))
                },
                JLToken::NumberFloat(f) => {
                    self.set_expect_after_member_value();
                    Ok((JPartialToken::MemberValue(JPValue::Float(f)), p))
                },
                JLToken::UnknownToken(s) => {
                    let errmsg = format!("Unknown token \"{}\" at position {}", s, p);
                    Err(JParserError::UnknownToken(errmsg))
                },
                _ => {
                    // Should not appear due to the concept of algorithm:
                    // JLToken::Whitespace, , JLToken::ArrayEnd,
                    // JLToken::NameSeparator, JLToken::ValueSeparator, JLToken::StringToken
                    println!("{ltk:?}");
                    panic!("Return this shit to developer!")
                },
            };
            println!("JPartialParser found: {:?} / Expecting: {:?}", token, self.expect);
            self.count += 1;
            token
        })
    }
}


pub struct JParser {
    partial_tokens: Vec<(JPartialToken, usize)>,
    // pub tokens ...
}

impl JParser {
    /// New type pattern, creates a new parser from given source.
    pub fn new(source: &str) -> JPResult<JParser> {
        let mut jpart_parser = JPartialParser::new(source);
        let mut partial_tokens: Vec<(JPartialToken, usize)> = Vec::new();
        for tk in jpart_parser {
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
        let mut parser = JPartialParser::new(r#"{"name": "Michael", "has_job": true, "has_kid": false, "pointer": null, "features": ["test", 10, true]}"#);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("name".to_string()), 3);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::from("Michael")), 11);
        assert_cmp!(parser, JPartialToken::MemberName("has_job".to_string()), 22);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::True), 32);
        assert_cmp!(parser, JPartialToken::MemberName("has_kid".to_string()), 39);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::False), 49);
        assert_cmp!(parser, JPartialToken::MemberName("pointer".to_string()), 57);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::Null), 67);
        assert_cmp!(parser, JPartialToken::MemberName("features".to_string()), 74);
        let array = vec![JPValue::String("test".to_string()), 
            JPValue::Integer(10), JPValue::True];
        assert_cmp!(parser, JPartialToken::Array(array), 98);
    }

    #[test]
    fn partial_parse_objects() {
        let mut parser = JPartialParser::new(r#"{"object": { "data": "data", "object2": {} } }"#);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 1);
        assert_cmp!(parser, JPartialToken::MemberName("object".to_string()), 3);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 12);
        assert_cmp!(parser, JPartialToken::MemberName("data".to_string()), 15);
        assert_cmp!(parser, JPartialToken::MemberValue(JPValue::from("data")), 23);
        assert_cmp!(parser, JPartialToken::MemberName("object2".to_string()), 31);
        assert_cmp!(parser, JPartialToken::ObjectBegin, 41);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 42);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 44);
        assert_cmp!(parser, JPartialToken::ObjectEnd, 46);
    }
}
