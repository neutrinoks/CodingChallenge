//! This is not an example, it is an utility to find errors on the Huffman-binary-tree during
//! development stage.

use cccompress::{
    Result,
    algorithm::*,
};

fn build_tree() -> Result<CtBinaryTree> {
    let content = std::fs::read_to_string("loremipsum.txt")?;
    let spec = CharSpectrum::from_stream(&content);
    Ok(CtBinaryTree::try_from(spec)?)
}

fn main() -> Result<()> {
    let _tree = build_tree()?;

    // Display the tree

    Ok(())
}
