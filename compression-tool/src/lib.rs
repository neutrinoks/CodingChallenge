//! Library with functionality of compression-tool.

pub mod algorithm;
pub mod command;
pub mod types;

use command::CtInput;
use algorithm::*;

pub use types::Result;

/// One of the internal development steps and functions to be tested.
fn frequency_analysis(input: &CtInput) -> Result<CharSpectrum> {
    Ok(CharSpectrum::from_stream(&input.content))
}

/// One of the internal development steps and functions to be tested.
fn create_huffman_tree(spectrum: CharSpectrum) -> Result<CtBinaryTree> {
    Ok(CtBinaryTree::try_from(spectrum)?)
}

/// One of the internal development steps and functions to be tested.
fn create_prefix_table(tree: &CtBinaryTree) -> PrefixCodeTable {
    PrefixCodeTable::from(tree)
}

/// Main entry method for compression-tool use case, to be able to separate the code into library
/// and not main module.
pub fn compression_tool(input: CtInput) -> Result<String> {
    let spectrum = frequency_analysis(&input)?;
    let h_tree = create_huffman_tree(spectrum)?;
    let table = create_prefix_table(&h_tree);
    Ok(format!("{table:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testfile(name: &str) -> CtInput {
        let args = crate::command::CtArgs {
            filename: name.to_string(),
        };
        CtInput::try_from(args).expect(&format!("testfile/expected: {}", name))
    }

    fn opendsa_example_spectrum() -> CharSpectrum {
        CharSpectrum(vec![
            ('z', 2),
            ('k', 7),
            ('m', 24),
            ('c', 32),
            ('u', 37),
            ('d', 42),
            ('l', 42),
            ('e', 120),
        ])
    }

    fn get_spectrum() -> CharSpectrum {
        frequency_analysis(&testfile("135-0.txt")).expect("frequency_analysis failed")
    }

    #[test]
    fn step_1() {
        let result = get_spectrum();
        let t = result
            .0
            .iter()
            .find(|&&x| x.0 == 't')
            .expect("no 't' found");
        let x = result
            .0
            .iter()
            .find(|&&x| x.0 == 'X')
            .expect("no 'X' found");
        assert_eq!(t.1, 223000);
        assert_eq!(x.1, 333);
    }

    #[test]
    fn step_2() {
        let spec = opendsa_example_spectrum();
        let tree = create_huffman_tree(spec).expect("create_huffman_tree failed");
        let mut tree_iter = tree.iter();

        assert!(tree_iter.next().unwrap().test_hierarchy(306));
        assert_eq!(*tree_iter.next().unwrap(), CtTreeNode::Bin('e', 120));
        assert!(tree_iter.next().unwrap().test_hierarchy(186));
        assert!(tree_iter.next().unwrap().test_hierarchy(79));
        assert_eq!(*tree_iter.next().unwrap(), CtTreeNode::Bin('u', 37));
        assert_eq!(*tree_iter.next().unwrap(), CtTreeNode::Bin('d', 42));
        assert!(tree_iter.next().unwrap().test_hierarchy(107));
        assert!(tree_iter.next().unwrap().test_bin('l', 42));
        assert!(tree_iter.next().unwrap().test_hierarchy(65));
        assert!(tree_iter.next().unwrap().test_bin('c', 32));
        assert!(tree_iter.next().unwrap().test_hierarchy(33));
        assert!(tree_iter.next().unwrap().test_hierarchy(9));
        assert!(tree_iter.next().unwrap().test_bin('z', 2));
        assert!(tree_iter.next().unwrap().test_bin('k', 7));
        assert!(tree_iter.next().unwrap().test_bin('m', 24));
    }

    #[test]
    fn step_3() {
        let spec = opendsa_example_spectrum();
        let tree = create_huffman_tree(spec).expect("create_huffman_tree failed");
        let prefix_table = create_prefix_table(&tree);

        let result = prefix_table.get(&'c').expect("no entry 'c' found");
        assert_eq!(*result, PrefixCodeEntry::test('c', 32, 14, 4));
        let result = prefix_table.get(&'d').expect("no entry 'd' found");
        assert_eq!(*result, PrefixCodeEntry::test('d', 42, 5, 3));
        let result = prefix_table.get(&'e').expect("no entry 'e' found");
        assert_eq!(*result, PrefixCodeEntry::test('e', 120, 0, 1));
        let result = prefix_table.get(&'k').expect("no entry 'k' found");
        assert_eq!(*result, PrefixCodeEntry::test('k', 7, 61, 6));
        let result = prefix_table.get(&'l').expect("no entry 'l' found");
        assert_eq!(*result, PrefixCodeEntry::test('l', 42, 6, 3));
        let result = prefix_table.get(&'m').expect("no entry 'm' found");
        assert_eq!(*result, PrefixCodeEntry::test('m', 24, 31, 5));
        let result = prefix_table.get(&'u').expect("no entry 'u' found");
        assert_eq!(*result, PrefixCodeEntry::test('u', 37, 4, 3));
        let result = prefix_table.get(&'z').expect("no entry 'z' found");
        assert_eq!(*result, PrefixCodeEntry::test('z', 2, 60, 6));
    }

    #[test]
    fn step_4() {
        todo!();
    }

    #[test]
    fn step_5() {
        todo!();
    }
}
