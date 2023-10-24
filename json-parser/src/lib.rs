//! A simple JSON-parser as a coding challenge by John Cricket.

// mod jdatatypes;
pub mod jlexer;
pub mod jparser;
pub mod jparser_types;

#[cfg(test)]
mod tests {
    use crate::{
        jlexer::JLexerToken as JLToken,
        jobject,
        jparser::{JParseError, JParser, JPartialExpect as JPExpect, UnexpTokenFeedb},
        jparser_types::{JMember, JObject, JPartialValue as JPValue, JValue},
        unexpected_token,
    };
    use totems::{assert_err, assert_ok};

    #[inline]
    fn expect_file(file: &str) -> String {
        std::fs::read_to_string(file).expect(&format!("missing test file {}", file))
    }

    #[test]
    fn cc_step_1() {
        let source = expect_file("tests/step1/valid.json");
        let mut parser = JParser::new(&source);
        assert_ok!(parser.parse(), value == JObject::default());

        let source = expect_file("tests/step1/invalid.json");
        let mut parser = JParser::new(&source);
        assert_err!(parser.parse(), value == JParseError::NoBeginningObject(1));
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
        let err = unexpected_token!(17, JLToken::ObjectEnd, &vec![JPExpect::MemberName]);
        assert_eq!(parser.parse(), err);

        let source = expect_file("tests/step2/invalid2.json");
        let mut parser = JParser::new(&source);
        let err = unexpected_token!(
            23,
            JLToken::UnknownToken("key".into()),
            &vec![JPExpect::MemberName]
        );
        assert_eq!(parser.parse(), err);
    }

    #[test]
    fn cc_step_3() {
        let source = expect_file("tests/step3/valid.json");
        let mut parser = JParser::new(&source);
        let obj = jobject!(
            "key1", JValue::from(true),
            "key2", JValue::from(false),
            "key3", JValue::from(JPValue::Null),
            "key4", JValue::from("value"),
            "key5", JValue::from(101)
            );
        assert_ok!(parser.parse(), value == obj);

        let source = expect_file("tests/step3/invalid.json");
        let mut parser = JParser::new(&source);
        let err = unexpected_token!(
            29,
            JLToken::UnknownToken("False".into()),
            &vec![JPExpect::MemberValue, JPExpect::ObjectBegin]
        );
        assert_eq!(parser.parse(), err);
    }

    #[test]
    fn cc_step_4() {
        let source = expect_file("tests/step4/valid.json");
        let mut parser = JParser::new(&source);
        let obj = jobject!(
            "key", JValue::from("value"),
            "key-n", JValue::from(false),
            "key-o", JValue::Object(JObject::default()),
            "key-l", JValue::Array(Vec::new())
            );
        assert_ok!(parser.parse(), value == obj);

        let source = expect_file("tests/step4/valid2.json");
        let mut parser = JParser::new(&source);
        let obj = jobject!(
            "key", JValue::from("value"),
            "key-n", JValue::from(101),
            "key-o", JValue::Object(jobject!("inner key", JValue::from("inner value"))),
            "key-l", JValue::Array(vec![JPValue::from("list value")])
            );
        assert_ok!(parser.parse(), value == obj);

        let source = expect_file("tests/step3/invalid.json");
        let mut parser = JParser::new(&source);
        let err = unexpected_token!(
            29,
            JLToken::UnknownToken("'".into()),
            &vec![JPExpect::MemberValue, JPExpect::ObjectBegin]
        );
        assert_eq!(parser.parse(), err);
    }
}
