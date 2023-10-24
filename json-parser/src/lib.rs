//! A simple JSON-parser as a coding challenge by John Cricket.

// mod jdatatypes;
pub mod jlexer;
pub mod jparser;
pub mod jparser_types;


#[cfg(test)]
mod tests {
    use totems::{assert_ok, assert_err};
    use crate::{
        jobject,
        jparser::{JParser, JParseError},
        jparser_types::{JPartialValue as JPValue, JValue, JObject, JMember},
    };

    #[inline]
    fn expect_file(file: &str) -> String {
        std::fs::read_to_string(file).expect(&format!("missing test file {}", file))
    }

    #[test]
    fn cc_step_1() {
        let source = expect_file("tests/step1/valid.json");
        let mut parser = JParser::new(&source);
        assert_ok!(parser.parse(), value == JObject::new());

        let source = expect_file("tests/step1/invalid.json");
        let mut parser = JParser::new(&source);
        assert_err!(parser.parse(), value == JParseError::NoBeginningObject);
    }

    #[test]
    fn cc_step_2() {
        let source = expect_file("tests/step2/valid.json");
        let mut parser = JParser::new(&source);
        let obj = jobject!("key", JValue::from("value"));
        assert_ok!(parser.parse(), value == obj);

        let source = expect_file("tests/step2/valid2.json");
        let mut parser = JParser::new(&source);
        let obj = jobject!("key", JValue::from("value"), "key2", JValue::from("value"));
        assert_ok!(parser.parse(), value == obj);

        let source = expect_file("tests/step2/invalid.json");
        let mut parser = JParser::new(&source);
        assert_err!(parser.parse(), value == JParseError::UnexpectedToken(_));

        let source = expect_file("tests/step2/invalid2.json");
        let mut parser = JParser::new(&source);
        assert_err!(parser.parse(), value == JParseError::UnexpectedToken(_));
    }
}
