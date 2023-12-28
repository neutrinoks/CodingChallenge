//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

use crate::algorithm::PrefixCodeTable;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// TODO Header type
///
/// **Byte Representation**
///
/// 0               (1) number of bytes (n) for an optional filename, 0 when no filename specified
/// 1..n+1          (2) optional filename
/// n+1..n+3        (3) number of bytes (m) for the prefix code table as u16 (2 Bytes)
/// n+3..n+m+3      (4) prefix code table
/// n+m+3..n+m+8    (5) 4 bytes u32, number of bytes of encoded data content
#[derive(Debug, PartialEq)]
pub struct Header {
    /// (Optional) specified filename.
    pub filename: String,
    /// The prefix code table.
    pub prefix_table: PrefixCodeTable,
    /// Number of bytes for the encoded data.
    pub data_bytes: u32,
}

impl Default for Header {
    fn default() -> Header {
        Header{
            filename: String::new(),
            prefix_table: PrefixCodeTable::default(),
            data_bytes: 0,
        }
    }
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
        let m = u16::from_le_bytes([data[n+1], data[n+2]]) as usize;
        let prefix_table = PrefixCodeTable::from(&data[n + 3..n + m + 3]);

        // (5)
        let idx = m + n + 3;
        let data_bytes = [data[idx], data[idx + 1], data[idx + 2], data[idx + 3]];
        let data_bytes = u32::from_le_bytes(data_bytes);

        Header {
            filename,
            prefix_table,
            data_bytes,
        }
    }
}

impl From<&Header> for Vec<u8> {
    fn from(hdr: &Header) -> Vec<u8> {
        let mut data = Vec::<u8>::new();

        // (1) & (2)
        if hdr.filename.is_empty() {
            data.push(0);
        } else {
            assert!(hdr.filename.len() < 256);
            data.push(hdr.filename.len() as u8);
            hdr.filename.chars().for_each(|c| data.push(c as u8));
        }

        // (3) & (4)
        let mut table_data = Vec::<u8>::from(&hdr.prefix_table);
        let m = (table_data.len() as u16).to_le_bytes();
        data.push(m[0]);
        data.push(m[1]);
        data.append(&mut table_data);

        // (5)
        let be_bytes = hdr.data_bytes.to_le_bytes();
        data.push(be_bytes[0]);
        data.push(be_bytes[1]);
        data.push(be_bytes[2]);
        data.push(be_bytes[3]);

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_to_bytes_1() {
        let header = Header {
            filename: String::new(),
            prefix_table: crate::tests::table_opendsa(),
            data_bytes: 1,
        };
        let output = Vec::<u8>::from(&header);
        assert_eq!(
            vec![
                0, 16, 0, 'e' as u8, 0u8, 'u' as u8, 4u8, 'd' as u8, 5u8, 'l' as u8, 6u8,
                'c' as u8, 14u8, 'm' as u8, 31u8, 'z' as u8, 60u8, 'k' as u8, 61u8, 1, 0, 0, 0,
            ],
            output
        );
    }

    #[test]
    fn header_to_bytes_2() {
        let header = Header {
            filename: "test".to_string(),
            prefix_table: crate::tests::table_opendsa(),
            data_bytes: 256,
        };
        let output = Vec::<u8>::from(&header);
        assert_eq!(
            vec![
                4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 16, 0, 'e' as u8, 0u8, 'u' as u8,
                4u8, 'd' as u8, 5u8, 'l' as u8, 6u8, 'c' as u8, 14u8, 'm' as u8, 31u8, 'z' as u8,
                60u8, 'k' as u8, 61u8, 0, 1, 0, 0,
            ],
            output
        );
    }

    #[test]
    fn header_to_bytes_and_back() {
        let header = Header {
            filename: "testfile.txt".to_string(),
            prefix_table: crate::tests::table_opendsa(),
            data_bytes: 1,
        };

        let data: Vec<u8> = Vec::from(&header);
        let output = Header::from(&data[..]);

        assert_eq!(header, output);
    }
}
