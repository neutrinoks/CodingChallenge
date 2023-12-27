//! Module contains several data type implementations, such as `CharSpectrum` or `CtBinaryTree`,
//! which represent milestone date outputs during the development.

use crate::algorithm::PrefixCodeTable;

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Some additional constant for additional validation, that we are reading the right file type.
/// Based on Illuminati-style.
pub const FILE_CONST: u8 = 23;

/// TODO Header type
///
/// **Byte Representation**
///
/// FILE_CONST  (0) File constant identifier, like specified above.
/// 1           (1) number of bytes (n) for an optional filename, 0 when no filename specified
/// 2..n+2      (2) optional filename
/// n+2..n+4    (3) number of bytes (m) for the prefix code table as u16 (2 Bytes)
/// n+4..n+m+4  (4) prefix code table
#[derive(Debug, PartialEq)]
pub struct Header {
    /// (Optional) specified filename.
    filename: String,
    /// The prefix code table.
    prefix_table: PrefixCodeTable,
}

impl TryFrom<&[u8]> for Header {
    type Error = ();

    fn try_from(data: &[u8]) -> std::result::Result<Header, Self::Error> {
        // (0)
        if data[0] != FILE_CONST {
            return Err(())
        }

        // (1) & (2)
        let n = data[1] as usize;
        let mut filename = String::new();
        if n > 0 {
            for &c in data.iter().skip(2).take(n) {
                filename.push(c as char);
            }
        }

        // (3) & (4)
        let m = ((data[n+2] as usize) << 8) | (data[n+3] as usize);
        let prefix_table = PrefixCodeTable::from(&data[n+4..n+m+4]);

        Ok(Header{ filename, prefix_table })
    }
}

impl From<&Header> for Vec<u8> {
    fn from(hdr: &Header) -> Vec<u8> {
        let mut data: Vec<u8> = vec![FILE_CONST];

        // (1) & (2)
        if hdr.filename.is_empty() {
            data.push(0);
        } else {
            data.push(hdr.filename.len() as u8);
            hdr.filename.as_bytes().iter().for_each(|x| data.push(*x));
        }

        // (3) & (4)
        let mut table_data = Vec::<u8>::from(&hdr.prefix_table);
        let m = table_data.len();
        data.push(((m & 0xff00) >> 8) as u8);
        data.push(((m & 0x00ff)) as u8);
        data.append(&mut table_data);

        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_to_vec() {
        let header = Header {
            filename: String::new(),
            prefix_table: crate::tests::table_opendsa(),
        };
        let output: Vec<u8> = Vec::from(&header);
        assert_eq!(
            vec![
                FILE_CONST,
                0,
                0, 16,
                'e' as u8, 0u8,
                'u' as u8, 4u8,
                'd' as u8, 5u8,
                'l' as u8, 6u8,
                'c' as u8, 14u8,
                'm' as u8, 31u8,
                'z' as u8, 60u8,
                'k' as u8, 61u8,
            ],
            output
            );
    }

    #[test]
    fn header_to_vec_and_back() {
        let header = Header {
            filename: String::new(),
            prefix_table: crate::tests::table_opendsa(),
        };

        let data: Vec<u8> = Vec::from(&header);
        let output = Header::try_from(&data[..]).expect("Header::try_from failed");

        assert_eq!(header, output);
    }

    #[test]
    fn write_and_read_again() {
        todo!()
    }
}
