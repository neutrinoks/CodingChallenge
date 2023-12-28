//! Module implements two types of bit-stream-iterators, one for reading and one for writing bit
//! streams. Those types are `BitStreamReader` and `BitStreamWriter`.

#[derive(Debug, Clone)]
pub struct BitStreamReader<'s> {
    /// The true data stream based on `Vec<u8>`.
    stream: &'s Vec<u8>,
    /// A bit-index.
    bidx: usize,
    /// A byte-index (char).
    cidx: usize,
}

impl<'s> BitStreamReader<'s> {
    pub fn new(stream: &'s Vec<u8>) -> BitStreamReader<'s> {
        BitStreamReader{
            stream,
            bidx: 0,
            cidx: 0,
        }
    }

    pub fn unused_bits(&self) -> usize {
        7 - self.bidx
    }
}


impl<'r> Iterator for BitStreamReader<'r> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cidx < self.stream.len() && self.bidx < 8 {
            let result = (self.stream[self.cidx] >> self.bidx) & 1u8;
            self.bidx += 1;
            if self.bidx == 8 {
                self.bidx = 0;
                self.cidx += 1;
            }
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

impl<'s> BitStreamWriter {
    pub fn new() -> BitStreamWriter {
        BitStreamWriter{
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

    pub fn finalize(mut self) -> (Vec<u8>, usize) {
        let uu_bits = if self.bidx == 0 {
            self.stream.pop();
            0
        } else {
            8 - self.bidx
        };
        (self.stream, uu_bits)
    }
}

impl Default for BitStreamWriter {
    fn default() -> BitStreamWriter {
        BitStreamWriter::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitstreamreader_simple() {
        let stream: Vec<u8> = vec![5, 6];
        let mut reader = BitStreamReader::new(&stream);

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
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
        assert_eq!(reader.next(), Some(0));
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
        // writer.add_bit(0);

        let (stream, uu_bits) = writer.finalize();
        assert_eq!(stream, vec![15u8, 64u8]);
        assert_eq!(uu_bits, 1);
    }
}
