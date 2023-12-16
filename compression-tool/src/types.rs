//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

use std::{borrow::Borrow, collections::HashMap};

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Stores a single frequency-bin, e.g. for the character 'a', how many times 'a' appeared in a
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
}

impl<'r> From<&'r CtBinaryTree> for CtBinaryTreeIter<'r> {
    fn from(tree: &'r CtBinaryTree) -> CtBinaryTreeIter<'r> {
        CtBinaryTreeIter {
            next_node: Some(&tree.node),
            parents: Vec::new(),
        }
    }
}

impl<'a> Iterator for CtBinaryTreeIter<'a> {
    type Item = &'a CtTreeNode;

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
