//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

use crate::bitstream::*;
use std::{borrow::Borrow, collections::HashMap};

/// Stores a single frequency-bin, e.g. for the character 'r', how many times 'r' appeared in a
/// given input stream.
#[derive(Debug, Eq, PartialEq)]
pub struct CharSpectrum(pub Vec<(char, usize)>);

impl CharSpectrum {
    /// Constructor from a given input text stream (`&str`).
    pub fn from_stream(stream: &str) -> CharSpectrum {
        let mut s = CharSpectrum(Vec::new());
        s.analyse_stream(stream);
        s
    }

    /// Analysis the given text stream and overwrites the internal char spectrum (by default it is
    /// empty). The generated spectrum will be sorted ascending by the bin's frequency.
    pub fn analyse_stream(&mut self, stream: &str) {
        let mut map: HashMap<char, usize> = HashMap::new();
        stream.chars().for_each(|c| {
            let cnt = if let Some(cnt) = map.get(&c) {
                cnt + 1
            } else {
                1
            };
            let _ = map.insert(c, cnt);
        });
        self.0 = map.into_iter().collect();
    }

    /// Sorts the internal array of char-frequency-bins ascending.
    pub fn sort(&mut self) {
        self.0.sort_by(|a, b| {
            if a.1 == b.1 {
                a.0.partial_cmp(&b.0).unwrap()
            } else {
                a.1.partial_cmp(&b.1).unwrap()
            }
        });
    }
}

/// Huffmann binary tree.
#[derive(Debug)]
pub struct CtBinaryTree {
    /// The actual binary data tree.
    node: Box<CtTreeNode>,
}

impl<'r> CtBinaryTree {
    pub fn iter(&'r self) -> CtBinaryTreeIter<'r> {
        CtBinaryTreeIter::from(self)
    }
}

impl From<CtTreeNode> for CtBinaryTree {
    fn from(node: CtTreeNode) -> CtBinaryTree {
        CtBinaryTree {
            node: Box::new(node),
        }
    }
}

impl TryFrom<CharSpectrum> for CtBinaryTree {
    type Error = String;

    fn try_from(cs: CharSpectrum) -> std::result::Result<CtBinaryTree, Self::Error> {
        // Step 1, convert each element from (char, usize) to CtTreeNode.
        if cs.0.is_empty() {
            return Err("CtBinaryTree::from(CharSpectrum) - empty CharSpectrum!".into());
        }
        let mut v_cs: Vec<CtTreeNode> = cs.0.into_iter().map(CtTreeNode::from).collect();

        // Step 2, iterate over array of nodes, build tree-nodes, until only one node is left.
        while v_cs.len() > 1 {
            let left = v_cs.remove(0);
            let right = v_cs.remove(0);
            let n_node = CtTreeNode::hierarchy(left, right);
            let pos = n_node.find_position_in(&v_cs);
            v_cs.insert(pos, n_node);
        }

        Ok(CtBinaryTree::from(v_cs.remove(0)))
    }
}

#[derive(Debug)]
pub struct CtBinaryTreeIter<'r> {
    next_node: Option<&'r CtTreeNode>,
    parents: Vec<(&'r CtTreeNode, bool)>,
    last_code: u8,
}

impl<'r> CtBinaryTreeIter<'r> {
    /// Generates the code from the given parents stack.
    pub fn last_code(&self) -> u8 {
        self.last_code
    }
}

impl<'r> From<&'r CtBinaryTree> for CtBinaryTreeIter<'r> {
    fn from(tree: &'r CtBinaryTree) -> CtBinaryTreeIter<'r> {
        CtBinaryTreeIter {
            next_node: Some(&tree.node),
            parents: Vec::new(),
            last_code: 0,
        }
    }
}

impl<'r> Iterator for CtBinaryTreeIter<'r> {
    type Item = &'r CtTreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next_node?;

        // The scheme reads as follow:
        // - Steps are describing what node to choose next from current node.
        // - Always try the highest rule, if not possible, try next.
        // Scheme:
        // 1. Left child
        // 2. Next findable Parent's right child
        match next {
            CtTreeNode::Hierarchy(_, left, _) => {
                self.parents.push((next, false));
                self.next_node = Some(left.as_ref().unwrap().borrow());
            }
            CtTreeNode::Bin(_, _) => {
                // In case of a new Bin, we update our internal code.
                let bits: Vec<u8> = self.parents.iter().map(|x| x.1 as u8).collect();
                let code: u8 = bits.iter().fold(0, |result, &bit| (result << 1) ^ bit);
                self.last_code = code;
                // Now we search for the next leaf.
                if self.parents.is_empty() {
                    self.next_node = None;
                } else {
                    let mut searching = true;
                    while searching {
                        let parent_right = if let Some(r) = self.parents.last() {
                            r.1
                        } else {
                            self.next_node = None;
                            break;
                        };
                        if parent_right {
                            // The parent's right path was iterated.
                            let _ = self.parents.pop();
                        } else {
                            // The parent's right path has not been iterated.
                            let parent_ref = self.parents.last_mut().unwrap();
                            match parent_ref.0 {
                                CtTreeNode::Hierarchy(_, _, right) => {
                                    self.next_node = Some(right.as_ref().unwrap().borrow());
                                }
                                _ => panic!("algorithm fail #1"),
                            }
                            parent_ref.1 = true;
                            searching = false;
                        }
                    }
                }
            }
        }

        Some(next)
    }
}

/// One single node of the Huffmann binary tree.
#[derive(Debug, PartialEq)]
pub enum CtTreeNode {
    /// A leaf/entry with a bin out of the `CharSpectrum`.
    Bin(char, usize),
    /// A hierarchy/sorting node with sum of contained bin-sums.
    Hierarchy(usize, Option<Box<CtTreeNode>>, Option<Box<CtTreeNode>>),
}

impl CtTreeNode {
    pub fn hierarchy(left: CtTreeNode, right: CtTreeNode) -> CtTreeNode {
        CtTreeNode::Hierarchy(
            left.frequency() + right.frequency(),
            Some(Box::new(left)),
            Some(Box::new(right)),
        )
    }

    pub fn frequency(&self) -> usize {
        match self {
            CtTreeNode::Bin(_, cnt) => *cnt,
            CtTreeNode::Hierarchy(sum, _, _) => *sum,
        }
    }

    pub fn find_position_in(&self, node_vec: &[CtTreeNode]) -> usize {
        let freq = self.frequency();
        let mut n_el = 0;
        for node in node_vec.iter() {
            if freq > node.frequency() {
                n_el += 1;
            } else {
                break;
            }
        }
        n_el
    }

    #[cfg(test)]
    pub fn test_hierarchy(&self, test_f: usize) -> bool {
        match self {
            CtTreeNode::Hierarchy(freq, _, _) => test_f == *freq,
            _ => false,
        }
    }

    #[cfg(test)]
    pub fn test_bin(&self, test_c: char, test_f: usize) -> bool {
        match self {
            CtTreeNode::Bin(c, f) => test_c == *c && test_f == *f,
            _ => false,
        }
    }
}

impl From<(char, usize)> for CtTreeNode {
    fn from(b: (char, usize)) -> CtTreeNode {
        CtTreeNode::Bin(b.0, b.1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrefixCodeEntry {
    /// Given letter, e.g. 'a'.
    pub letter: char,
    /// The derived code for this letter by using the Huffman-binary-tree.
    pub code: u8,
    /// How many bits are needed for the derived code of this letter.
    pub bits: usize,
}

impl PrefixCodeEntry {
    /// New type pattern as constructor for `PrefixCodeEntry`.
    pub fn new(letter: char, code: u8) -> PrefixCodeEntry {
        PrefixCodeEntry {
            letter,
            code,
            bits: count_bits(code),
        }
    }

    #[cfg(test)]
    pub fn test(letter: char, code: u8, bits: usize) -> PrefixCodeEntry {
        PrefixCodeEntry { letter, code, bits }
    }
}

impl std::fmt::Display for PrefixCodeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "| {} | {:3} |", self.letter, self.code)
    }
}

fn count_bits(code: u8) -> usize {
    let mut bits = 1; // in case of zero
    for n in (0..8).rev() {
        if code & (1 << n) > 0 {
            bits = n + 1;
            break;
        }
    }
    bits
}

/// TODO
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PrefixCodeTable(pub Vec<PrefixCodeEntry>);

impl<'a> PrefixCodeTable {
    pub fn get_by_char(&'a self, c: char) -> Option<&'a PrefixCodeEntry> {
        self.0.iter().find(|&e| e.letter == c)
    }

    pub fn get_by_code(&'a self, c: u8) -> Option<&'a PrefixCodeEntry> {
        self.0.iter().find(|&e| e.code == c)
    }

    pub fn iter(&'a self) -> PrefixCodeTableIter<'a> {
        PrefixCodeTableIter {
            table: &self.0,
            idx: 0,
        }
    }

    pub fn code(&self, c: char) -> Option<u8> {
        self.get_by_char(c).map(|e| e.code)
    }

    pub fn letter(&self, c: u8) -> Option<char> {
        self.get_by_code(c).map(|e| e.letter)
    }

    pub fn text2stream(&self, text: &str) -> crate::Result<(Vec<u8>, u8)> {
        let mut stream = BitStreamWriter::new();

        for c in text.chars() {
            let e = if let Some(e) = self.get_by_char(c) {
                e
            } else {
                return Err(format!("no entry with letter '{}'", c).into());
            };

            let bits = to_bits(e.code);
            for b in bits[0..e.bits].iter().rev() {
                stream.add_bit(*b);
            }
        }

        Ok(stream.finalize())
    }

    pub fn stream2text(&self, _stream: &[u8]) -> String {
        todo!();
    }
}

impl std::ops::Index<usize> for PrefixCodeTable {
    type Output = PrefixCodeEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl From<&CtBinaryTree> for PrefixCodeTable {
    fn from(tree: &CtBinaryTree) -> PrefixCodeTable {
        let mut table: Vec<PrefixCodeEntry> = Vec::new();
        let mut tree_iter = tree.iter();
        let mut searching = true;

        let mut count = 0;
        while searching {
            if let Some(node) = tree_iter.next() {
                match node {
                    CtTreeNode::Bin(c, _f) => {
                        let code = tree_iter.last_code();
                        table.push(PrefixCodeEntry::new(*c, code));
                        count += 1;
                    }
                    _ => continue,
                }
            } else {
                searching = false;
            }
        }
        table.sort_by(|x, y| x.code.cmp(&y.code));
        println!("PrefixCodeTable generated with {} entries", count);

        PrefixCodeTable(table)
    }
}

impl From<&[u8]> for PrefixCodeTable {
    fn from(data: &[u8]) -> PrefixCodeTable {
        let mut table: Vec<PrefixCodeEntry> = Vec::new();
        for i in (0..data.len()).step_by(2) {
            table.push(PrefixCodeEntry::new(data[i] as char, data[i + 1]));
        }
        table.sort_by(|x, y| x.code.cmp(&y.code));

        PrefixCodeTable(table)
    }
}

impl From<&PrefixCodeTable> for Vec<u8> {
    fn from(table: &PrefixCodeTable) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in table.iter() {
            data.push(entry.letter as u8);
            data.push(entry.code);
        }
        data
    }
}

#[derive(Debug, Clone)]
pub struct PrefixCodeTableIter<'a> {
    table: &'a Vec<PrefixCodeEntry>,
    idx: usize,
}

impl<'a> Iterator for PrefixCodeTableIter<'a> {
    type Item = &'a PrefixCodeEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.table.len() {
            let val = &self.table[self.idx];
            self.idx += 1;
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PrefixCodeEntry, PrefixCodeTable};

    #[test]
    fn pfc_iter() {
        let table = crate::tests::table_opendsa();
        let mut table_iter = table.iter();
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('e', 0)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('u', 4)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('d', 5)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('l', 6)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('c', 14)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('m', 31)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('z', 60)));
        assert_eq!(table_iter.next(), Some(&PrefixCodeEntry::new('k', 61)));
    }

    #[test]
    fn pfc_to_bytes() {
        let table = crate::tests::table_opendsa();
        let data = Vec::<u8>::from(&table);
        assert_eq!(
            vec![
                'e' as u8, 0u8, 'u' as u8, 4u8, 'd' as u8, 5u8, 'l' as u8, 6u8, 'c' as u8, 14u8,
                'm' as u8, 31u8, 'z' as u8, 60u8, 'k' as u8, 61u8,
            ],
            data
        );
    }

    #[test]
    fn pfc_to_bytes_and_back() {
        let table = crate::tests::table_opendsa();
        let data = Vec::<u8>::from(&table);
        let result = PrefixCodeTable::from(&data[..]);
        assert_eq!(table, result);
    }

    #[test]
    fn pfc_text2stream() {
        let table = crate::tests::table_opendsa();
        let (stream, uu_bits) = table
            .text2stream("lude")
            .expect("PrefixCodeTable::text2stream() failed");
        // "lude" -> 6, 4, 5, 0 -> 110, 100, 101, 0 -> 0101001011
        // "lude" -> 01 0100.1011 -> 75, 1
        println!("{stream:?}");
        for b in stream.iter() {
            println!("{:#b}", b);
        }
        assert_eq!(stream, vec![75u8, 1u8]);
        assert_eq!(uu_bits, 6);
    }
}
