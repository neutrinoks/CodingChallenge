//! Library with functionality of compression-tool.

mod algorithm;
mod bitstream;
mod command;
mod fs;
mod types;

use algorithm::*;

pub use command::CtInput;
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

    let mut output = "Abstract from the PrefixCodeTable\n".to_string();
    for i in 0..20 {
        output.push_str(&format!("{}\n", table[i]));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Header;

    pub(crate) fn testfile(name: &str) -> CtInput {
        let args = crate::command::CtArgs {
            source: name.to_string(),
            filename: None,
        };
        CtInput::try_from(args).expect(&format!("testfile/expected: {}", name))
    }

    pub(crate) fn spec_opendsa() -> CharSpectrum {
        // Example from https://opendsa-server.cs.vt.edu/ODSA/Books/CS3/html/Huffman.html
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

    pub(crate) fn spec_135_0() -> CharSpectrum {
        frequency_analysis(&testfile("135-0.txt")).expect("frequency_analysis failed")
    }

    pub(crate) fn table_opendsa() -> PrefixCodeTable {
        let mut table: Vec<PrefixCodeEntry> = Vec::new();
        table.push(PrefixCodeEntry::new('e', 0));
        table.push(PrefixCodeEntry::new('u', 4));
        table.push(PrefixCodeEntry::new('d', 5));
        table.push(PrefixCodeEntry::new('l', 6));
        table.push(PrefixCodeEntry::new('c', 14));
        table.push(PrefixCodeEntry::new('m', 31));
        table.push(PrefixCodeEntry::new('z', 60));
        table.push(PrefixCodeEntry::new('k', 61));
        PrefixCodeTable(table)
    }

    // pub(crate) fn table_135_0() -> PrefixCodeTable {
    //     let tree = create_huffman_tree(spec_135_0()).unwrap();
    //     create_prefix_table(&tree)
    // }

    #[test]
    fn step_1() {
        let result = spec_135_0();
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
        let spec = spec_opendsa();
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
        let spec = spec_opendsa();
        let tree = create_huffman_tree(spec).expect("create_huffman_tree failed");
        let prefix_table = create_prefix_table(&tree);

        let result = prefix_table.get_by_char('c').expect("no entry 'c' found");
        assert_eq!(*result, PrefixCodeEntry::test('c', 14, 4));
        let result = prefix_table.get_by_char('d').expect("no entry 'd' found");
        assert_eq!(*result, PrefixCodeEntry::test('d', 5, 3));
        let result = prefix_table.get_by_char('e').expect("no entry 'e' found");
        assert_eq!(*result, PrefixCodeEntry::test('e', 0, 1));
        let result = prefix_table.get_by_char('k').expect("no entry 'k' found");
        assert_eq!(*result, PrefixCodeEntry::test('k', 61, 6));
        let result = prefix_table.get_by_char('l').expect("no entry 'l' found");
        assert_eq!(*result, PrefixCodeEntry::test('l', 6, 3));
        let result = prefix_table.get_by_char('m').expect("no entry 'm' found");
        assert_eq!(*result, PrefixCodeEntry::test('m', 31, 5));
        let result = prefix_table.get_by_char('u').expect("no entry 'u' found");
        assert_eq!(*result, PrefixCodeEntry::test('u', 4, 3));
        let result = prefix_table.get_by_char('z').expect("no entry 'z' found");
        assert_eq!(*result, PrefixCodeEntry::test('z', 60, 6));
    }

    #[test]
    fn step_4() {
        let fname = "testfile.cpd";
        let header = Header {
            filename: String::new(),
            prefix_table: crate::tests::table_opendsa(),
            data_bytes: 0,
        };
        let data: Vec<u8> = Vec::from(&header);
        std::fs::write(fname, &data[..]).expect("file writing failed");

        let data: Vec<u8> = std::fs::read(fname).expect("file read failed");
        let result = Header::try_from(&data[..]).expect("Header::try_from failed");

        assert_eq!(header, result);
    }

    #[test]
    fn step_5() {
        todo!();
        // translate input text into byte-stream by using PrefixCodeTable
    }

    #[test]
    fn step_6() {
        todo!();
        // read in a header from a file and decode the byte-stream back to text.
    }

    #[test]
    fn step_7() {
        todo!();
    }
}
