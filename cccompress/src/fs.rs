//! Module contains read and write operations related to files on harddisk, to simplify and
//! generalize reading and writing from and to files.

use crate::types::{Header, Result};
use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

/// File extension, file type specification.
pub const FILE_EXTENSION: &str = "cpd";

/// Some additional constant for additional validation, that we are reading the right file type.
/// Based on Illuminati-style.
pub const FILE_CONST: u8 = 23;

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
pub fn write(filename: &str, header: &Header, data: &[u8]) -> Result<usize> {
    check_filename(filename)?;
    if data.len() != (header.data_bytes as usize) {
        return Err(format!("write: header expects {} bytes, but data has {}",
                           header.data_bytes, data.len()).into())
    }

    let mut bytes = 0;
    let mut file = File::create(filename)?;
    let mut buffer = Vec::<u8>::new();

    // Initially we write the FILE_CONST as identifier of the correct file format.
    buffer.push(FILE_CONST);

    // Followed by the length of the header (LE) and the header itself.
    let mut hdr_data = Vec::<u8>::from(header);
    let hdr_len = hdr_data.len() as u32;
    hdr_len.to_le_bytes().iter().for_each(|b| buffer.push(*b));
    buffer.append(&mut hdr_data);

    // Followed by the data content.
    buffer.extend_from_slice(data);

    bytes += file.write(&buffer[..])?;
    file.flush()?;

    Ok(bytes)
}

/// TODO
pub fn read(filename: &str, header: &mut Header, data: &mut Vec<u8>) -> Result<usize> {
    check_filename(filename)?;

    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::<u8>::new();
    let mut bytes = 0;

    bytes += reader.read_to_end(&mut buffer)?;

    // Same like above...
    if buffer[0] != FILE_CONST {
        return Err("no file constant detected, maybe another file type?"
            .to_string()
            .into());
    }

    // Same like above...
    let hdr_le_bytes = [buffer[1], buffer[2], buffer[3], buffer[4]];
    let hdr_len = u32::from_le_bytes(hdr_le_bytes) as usize;
    *header = Header::try_from(&buffer[5..5 + hdr_len])?;

    // Same like above...
    if (header.data_bytes as usize) != buffer.len() - 5 - hdr_len {
        return Err(format!("'{filename}' seems to be broken, header expects {} data bytes, but only {} remain",
                           header.data_bytes, buffer.len() - 5 - hdr_len).into())
    }
    data.extend_from_slice(&buffer[5+hdr_len..]);

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let fname = "testfile.cpd";
        let header = Header {
            filename: "othername.txt".to_string(),
            prefix_table: crate::tests::table_opendsa(),
            data_bytes: 4,
            unused_bits: 3,
        };
        let data: Vec<u8> = vec![1, 2, 3, 4];

        let bytes_wr = write(&fname, &header, &data[..]).expect("write() failed");
        assert!(std::path::Path::new(fname).exists());

        let mut res_hdr = Header::default();
        let mut res_dat = Vec::<u8>::new();
        let bytes_rd = read(&fname, &mut res_hdr, &mut res_dat).expect("read() failed");
        assert_eq!(bytes_wr, bytes_rd);
        assert_eq!(header, res_hdr);
        assert_eq!(data, res_dat);

        std::fs::remove_file(fname).expect("removing testfile failed");
    }
}
