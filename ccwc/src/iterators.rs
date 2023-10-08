//! Module encapsules individual iterator implementations.

/// Iterator for extracting words out of a text properly.
#[derive(Clone, Debug)]
pub struct WordIterator<'r> {
    /// Source text to be iterated.
    text: &'r str,
    /// Internal iterator.
    iter: std::str::CharIndices<'r>,
}

impl<'r> WordIterator<'r> {
    pub fn new(text: &'r str) -> WordIterator<'r> {
        let iter = text.char_indices();
        WordIterator { text, iter }
    }
}

impl<'r> Iterator for WordIterator<'r> {
    type Item = &'r str;

    fn next(&mut self) -> Option<&'r str> {
        let mut set = false;
        let mut start = 0;

        // Step 1: Search for next beginning word.
        for (i, c) in self.iter.by_ref() {
            if c.is_alphanumeric() {
                start = i;
                set = true;
                break;
            }
        }
        if !set {
            return None;
        }

        // Step 2: Search for end of this word.
        let mut stop = start;
        for (i, c) in self.iter.by_ref() {
            stop = i;
            if !(c.is_alphanumeric() || c == '-' || c == '.') {
                break;
            }
        }

        Some(&self.text[start..stop])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn worditer_simple_line() {
        let text: &str = "This is a simple, single line of text.";
        let iter = WordIterator::new(text);
        assert_eq!(iter.count(), 8);
    }

    #[test]
    fn worditer_special_characters() {
        let text: &str = "\u{feff}This is a simple,\nvery simple\t line of text.";
        let iter = WordIterator::new(text);
        assert_eq!(iter.count(), 9);
    }
}
