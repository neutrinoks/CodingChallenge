//! Module implements two types of bit-stream-iterators, one for reading and one for writing bit
//! streams. Those types are `BitStreamReader` and `BitStreamWriter`.

#[derive(Debug, Clone)]
pub struct BitStreamReader<'s> {
    /// The true data stream based on `Vec<u8>`.
    stream: &'s [u8],
    /// Number of unused bits.
    last_bit: usize,
    /// A bit-index.
    bidx: usize,
    /// A byte-index (char).
    cidx: usize,
}

impl<'s> BitStreamReader<'s> {
    pub fn new(stream: &'s [u8], uu_bits: u8) -> BitStreamReader<'s> {
        BitStreamReader {
            stream,
            last_bit: 8 - (uu_bits as usize),
            bidx: 0,
            cidx: 0,
        }
    }

    fn inc_counter(&mut self) {
        self.bidx += 1;
        if self.bidx == 8 {
            self.bidx = 0;
            self.cidx += 1;
        }
    }

    fn still_going(&self) -> bool {
        if self.cidx < self.stream.len() {
            if self.cidx + 1 == self.stream.len() {
                self.bidx < self.last_bit
            } else {
                true
            }
        } else {
            false
        }
    }
}

impl<'r> Iterator for BitStreamReader<'r> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.still_going() {
            let result = (self.stream[self.cidx] >> self.bidx) & 1u8;
            self.inc_counter();
            Some(result)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct BitStreamWriter {
    /// The true data stream based on `Vec<u8>`.
    stream: Vec<u8>,
    /// A bit-index.
    bidx: usize,
    /// A byte-index (char).
    cidx: usize,
}

impl BitStreamWriter {
    pub fn new() -> BitStreamWriter {
        BitStreamWriter {
            stream: vec![0],
            bidx: 0,
            cidx: 0,
        }
    }

    pub fn add_bit(&mut self, bit: u8) {
        if bit != 0 {
            self.stream[self.cidx] |= 1 << self.bidx;
        }
        self.bidx += 1;
        if self.bidx == 8 {
            self.stream.push(0);
            self.bidx = 0;
            self.cidx += 1;
        }
    }

    pub fn finalize(mut self) -> (Vec<u8>, u8) {
        let uu_bits = if self.bidx == 0 {
            self.stream.pop();
            0
        } else {
            8 - (self.bidx as u8)
        };
        (self.stream, uu_bits)
    }
}

impl Default for BitStreamWriter {
    fn default() -> BitStreamWriter {
        BitStreamWriter::new()
    }
}

pub fn to_bits(code: u8) -> [u8; 8] {
    [
        code & 1u8,
        (code >> 1) & 1u8,
        (code >> 2) & 1u8,
        (code >> 3) & 1u8,
        (code >> 4) & 1u8,
        (code >> 5) & 1u8,
        (code >> 6) & 1u8,
        (code >> 7) & 1u8,
    ]
}

#[derive(Debug)]
pub struct BitBuffer {
    val: u8,
    cnt: usize,
}

impl BitBuffer {
    pub fn new() -> BitBuffer {
        BitBuffer{val: 0, cnt: 0}
    }

    pub fn reset(&mut self) {
        self.val = 0;
        self.cnt = 0;
    }

    pub fn add_bit(&mut self, bit: u8) {
        self.cnt += 1;
        assert!(self.cnt <= 8);
        self.val = (self.val << 1) | bit;
    }
}

impl std::ops::Deref for BitBuffer {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitstreamreader_simple() {
        let stream: Vec<u8> = vec![5, 6];
        let mut reader = BitStreamReader::new(&stream, 5u8);

        assert_eq!(reader.next(), Some(1));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(1));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));

        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(1));
        assert_eq!(reader.next(), Some(1));
        assert_eq!(reader.next(), None);
    }

    #[test]
    fn bitstreamwriter_simple() {
        let mut writer = BitStreamWriter::new();

        writer.add_bit(1);
        writer.add_bit(1);
        writer.add_bit(1);
        writer.add_bit(1);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);

        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(0);
        writer.add_bit(1);
        // one bit missing

        let (stream, uu_bits) = writer.finalize();
        assert_eq!(stream, vec![15u8, 64u8]);
        assert_eq!(uu_bits, 1);
    }
}
