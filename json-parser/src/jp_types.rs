//! TODO

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
