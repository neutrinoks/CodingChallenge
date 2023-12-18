//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// TODO Header type
#[derive(Debug)]
pub struct Header {
    /// Number of bytes for the prefix code table.
    prefix_table_size: usize,
}
