//! A simple JSON-parser as a coding challenge by John Cricket.

// mod jdatatypes;
pub mod jlexer;
pub mod jparser;
pub mod jparser_types;


#[cfg(test)]
mod tests {
    use totems::{assert_ok, assert_err};
    use crate::jparser::{JObject, JParser, JParseError};

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
}
