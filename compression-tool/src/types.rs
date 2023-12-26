//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

use crate::algorithm::PrefixCodeTable;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// TODO Header type
///
/// **Byte Representation**
///
/// 0           (1) number of bytes (n) for an optional filename, 0 when no filename specified
/// 1..n+1      (2) optional filename
/// n+1..n+3    (3) number of bytes (m) for the prefix code table as u16 (2 Bytes)
/// n+3..n+m+3  (4) prefix code table
#[derive(Debug, PartialEq)]
pub struct Header {
    /// (Optional) specified filename.
    filename: String,
    /// The prefix code table.
    prefix_table: PrefixCodeTable,
}

impl From<&[u8]> for Header {
    fn from(data: &[u8]) -> Header {
        // (1) & (2)
        let n = data[0] as usize;
        let mut filename = String::new();
        if n > 0 {
            for &c in data.iter().skip(1).take(n) {
                filename.push(c as char);
            }
        }

        // (3) & (4)
        let m = ((data[n+1] as usize) << 8) & (data[n+2] as usize);
        let prefix_table = PrefixCodeTable::from(&data[n+3..n+m+3]);

        Header{ filename, prefix_table }
    }
}

impl From<&Header> for Vec<u8> {
    fn from(hdr: &Header) -> Vec<u8> {
        let mut data: Vec<u8> = Vec::new();

        // (1) & (2)
        if hdr.filename.is_empty() {
            data.push(0);
        } else {
            data.push(hdr.filename.len() as u8);
            hdr.filename.as_bytes().iter().for_each(|x| data.push(*x));
        }

        // (3) & (4)
        let table_iter = hdr.prefix_table.iter();
        let n_bytes = table_iter.clone().count() * 2;
        data.push(((n_bytes & 0xff00) >> 8) as u8);
        data.push((n_bytes & 0xff) as u8);
        for entry in table_iter {
            data.push(entry.letter as u8);
            data.push(entry.code);
        }

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_to_vec_and_back() {
        let header = Header {
            filename: String::new(),
            prefix_table: crate::tests::table_opendsa(),
        };

        let data: Vec<u8> = Vec::from(&header);
        let output = Header::from(&data[..]);

        assert_eq!(header, output);
    }

    #[test]
    fn write_and_read_again() {
        todo!()
    }
}
