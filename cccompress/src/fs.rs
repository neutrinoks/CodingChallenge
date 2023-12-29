//! Module contains read and write operations related to files on harddisk, to simplify and
//! generalize reading and writing from and to files.

use std::{
    fs::File,
    io::{prelude::*, BufReader},
};
use crate::{
    Result,
    algorithm::PrefixCodeTable,
};

/// File extension, file type specification.
pub const FILE_EXTENSION: &str = "cpd";

/// Some additional constant for additional validation, that we are reading the right file type.
/// Based on Illuminati-style.
pub const FILE_CONST: u8 = 23;

/// TODO Header type
///
/// **Byte Representation**
///
/// 0               (1) number of bytes (n) for an optional filename, 0 when no filename specified
/// 1..n+1          (2) optional filename
/// n+1..n+3        (3) number of bytes (m) for the prefix code table as u16 (2 Bytes)
/// n+3..n+m+3      (4) prefix code table
/// n+m+3..n+m+8    (5) 4 bytes u32, number of bytes of encoded data content
/// n+m+8           (6) number of unused bits in the last byte
#[derive(Debug, PartialEq)]
pub struct Header {
    /// (Optional) specified filename.
    pub filename: String,
    /// The prefix code table.
    pub prefix_table: PrefixCodeTable,
    /// Number of bytes for the encoded data.
    pub data_bytes: u32,
    /// Number of unused bits at the last byte.
    pub unused_bits: u8,
}

impl Default for Header {
    fn default() -> Header {
        Header{
            filename: String::new(),
            prefix_table: PrefixCodeTable::default(),
            data_bytes: 0,
            unused_bits: 0,
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

        // (6)
        let idx = idx + 4;
        let unused_bits = data[idx];

        Header {
            filename,
            prefix_table,
            data_bytes,
            unused_bits,
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

        // (6)
        data.push(hdr.unused_bits);

        data
    }
}

/// Method checks for a correct filename ending regarding the file type extension. Therefor, see
/// the contant `FILE_EXTENSION`.
fn check_filename(name: &str) -> Result<()> {
    if name.ends_with(&format!(".{}", FILE_EXTENSION)) {
        Ok(())
    } else {
        Err(format!("'{}' does not end with '.{}'", name, FILE_EXTENSION).into())
    }
}

/// TODO
#[derive(Debug, PartialEq)]
pub struct CompressedData {
    /// The header of the compressed file.
    pub header: Header,
    /// The data of the compressed file.
    pub data: Vec<u8>,
}

impl CompressedData {
    /// TODO
    pub fn write(&self, filename: &str) -> Result<usize> {
        check_filename(filename)?;
        if self.data.len() != (self.header.data_bytes as usize) {
            return Err(format!("write: header expects {} bytes, but data has {}",
                               self.header.data_bytes, self.data.len()).into())
        }

        let mut bytes = 0;
        let mut file = File::create(filename)?;
        let mut buffer = Vec::<u8>::new();

        // Initially we write the FILE_CONST as identifier of the correct file format.
        buffer.push(FILE_CONST);

        // Followed by the length of the header (LE) and the header itself.
        let mut hdr_data = Vec::<u8>::from(&self.header);
        let hdr_len = hdr_data.len() as u32;
        hdr_len.to_le_bytes().iter().for_each(|b| buffer.push(*b));
        buffer.append(&mut hdr_data);

        // Followed by the data content.
        buffer.extend_from_slice(&self.data[..]);

        bytes += file.write(&buffer[..])?;
        file.flush()?;

        Ok(bytes)
    }

    /// TODO
    pub fn read(filename: &str) -> Result<CompressedData> {
        check_filename(filename)?;

        let file = File::open(filename)?;
        let mut reader = BufReader::new(file);
        let mut buffer = Vec::<u8>::new();

        reader.read_to_end(&mut buffer)?;

        // Same like above...
        if buffer[0] != FILE_CONST {
            return Err("no file constant detected, maybe another file type?"
                .to_string()
                .into());
        }

        // Same like above...
        let hdr_le_bytes = [buffer[1], buffer[2], buffer[3], buffer[4]];
        let hdr_len = u32::from_le_bytes(hdr_le_bytes) as usize;
        let header = Header::try_from(&buffer[5..5 + hdr_len])?;

        // Same like above...
        if (header.data_bytes as usize) != buffer.len() - 5 - hdr_len {
            return Err(format!("'{filename}' seems to be broken, header expects {} data bytes, but only {} remain",
                               header.data_bytes, buffer.len() - 5 - hdr_len).into())
        }
        let mut data = Vec::<u8>::new();
        data.extend_from_slice(&buffer[5+hdr_len..]);

        Ok(CompressedData{
            header,
            data,
        })
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
            unused_bits: 4,
        };
        let output = Vec::<u8>::from(&header);
        assert_eq!(
            vec![
                0, 16, 0, 'e' as u8, 0u8, 'u' as u8, 4u8, 'd' as u8, 5u8, 'l' as u8, 6u8,
                'c' as u8, 14u8, 'm' as u8, 31u8, 'z' as u8, 60u8, 'k' as u8, 61u8, 1, 0, 0, 0, 4,
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
            unused_bits: 3,
        };
        let output = Vec::<u8>::from(&header);
        assert_eq!(
            vec![
                4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 16, 0, 'e' as u8, 0u8, 'u' as u8,
                4u8, 'd' as u8, 5u8, 'l' as u8, 6u8, 'c' as u8, 14u8, 'm' as u8, 31u8, 'z' as u8,
                60u8, 'k' as u8, 61u8, 0, 1, 0, 0, 3,
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
            unused_bits: 7,
        };

        let data: Vec<u8> = Vec::from(&header);
        let output = Header::from(&data[..]);

        assert_eq!(header, output);
    }

    #[test]
    fn write_and_read() {
        let fname = "testfile.cpd";
        let cdata = CompressedData{
            header: Header {
                filename: "othername.txt".to_string(),
                prefix_table: crate::tests::table_opendsa(),
                data_bytes: 4,
                unused_bits: 3,
            },
            data: vec![1, 2, 3, 4],
        };

        cdata.write(&fname).expect("write() failed");
        assert!(std::path::Path::new(fname).exists());

        let res_cdata = CompressedData::read(&fname).expect("read() failed");
        assert_eq!(cdata, res_cdata);

        std::fs::remove_file(fname).expect("removing testfile failed");
    }
}
