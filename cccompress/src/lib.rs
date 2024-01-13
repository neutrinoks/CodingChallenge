//! Library with functionality of compression-tool.

mod command;
pub mod fs;

use huffman_coding::{HuffmanReader, HuffmanTree, HuffmanWriter};
use std::io::{Cursor, Read, Write};

pub use command::CtDirective;
use fs::{CompressedData, Header};

/// Crate common default Result type.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Encoding method to transform text into encoded, compressed bit stream.
fn compress(text: &str) -> Result<CompressedData> {
    let tree = HuffmanTree::from_data(text.as_bytes());
    let table = Vec::<u8>::from(tree.to_table());

    let mut data = Vec::new();
    {
        let mut writer = HuffmanWriter::new(&mut data, &tree);
        let _ = writer.write(text.as_bytes())?;
    }
    let len = data.len() as u32;

    Ok(CompressedData {
        header: Header {
            filename: String::new(),
            prefix_table: table,
            data_bytes: len,
        },
        data,
    })
}

/// Decoding method to transform encoded, compressed bit stream back to text.
fn decompress(cdata: &CompressedData) -> Result<String> {
    let tree = HuffmanTree::from_table(&cdata.header.prefix_table[..]);
    let cursor = Cursor::new(&cdata.data[..]);

    let mut data = Vec::<u8>::new();
    let mut reader = HuffmanReader::new(cursor, tree);

    let bytes = reader.read_to_end(&mut data)?;
    println!("HuffmanReader read {bytes} Bytes");

    Ok(String::from_utf8(data)?)
}

/// Main entry method for compression-tool use case, to be able to separate the code into library
/// and not main module.
pub fn compression_tool(directive: CtDirective) -> Result<String> {
    Ok(match directive {
        CtDirective::Pack(source, of) => {
            let content = std::fs::read_to_string(&source)?;
            let fname = if let Some(ofname) = of {
                ofname
            } else {
                fs::switch_file_type(&source)
            };

            let cdata = compress(&content)?;
            let bytes = cdata.write(&fname)?;

            format!("Compressed '{source}'. Wrote {bytes} bytes to '{fname}'")
        }
        CtDirective::Unpack(source) => {
            let cdata = CompressedData::read(&source)?;
            let fname = if cdata.header.filename.is_empty() {
                fs::switch_file_type(&source)
            } else {
                cdata.header.filename.clone()
            };

            let text = decompress(&cdata)?;
            std::fs::write(&fname, &text)?;
            let bytes = text.len();

            format!("Decompressed '{source}'. Wrote {bytes} bytes to '{fname}'")
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn testfile(name: &str) -> String {
        std::fs::read_to_string(name).expect(&format!("could not open testfile '{name}'"))
    }

    #[test]
    fn encode_decode_testfile() {
        let input = testfile("135-0.txt");
        let cdata = compress(&input).expect("compress() failed");
        let output = decompress(&cdata).expect("decompress() failed");
        assert_eq!(input, output);
    }

    #[test]
    fn write_read_file() {
        let fname = "135-0.txt";
        let input = testfile(fname);
        let cdata = compress(&input).expect("compress() failed");

        let fname = fs::switch_file_type(&fname);
        println!("{fname:}");
        cdata.write(&fname).expect("CompressedData::write() failed");

        let cdata = CompressedData::read(&fname).expect("CompressedData::read() failed");

        let output = decompress(&cdata).expect("decompress() failed");
        assert_eq!(input, output);

        std::fs::remove_file(&fname).expect("removing testfile failed");
    }
}
