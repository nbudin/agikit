use std::iter::Peekable;

use bitfield_struct::bitfield;

#[bitfield(u8)]
struct RLEColorByte {
    #[bits(4)]
    count: u8,
    #[bits(4)]
    color: u8,
}

pub struct ViewRLEDecoder<'a> {
    input: &'a mut dyn Iterator<Item = u8>,
    current_color: u8,
    counter: usize,
}

impl<'a> ViewRLEDecoder<'a> {
    pub fn new(input: &'a mut dyn Iterator<Item = u8>) -> Self {
        ViewRLEDecoder {
            input,
            current_color: 0,
            counter: 0,
        }
    }
}

impl<'a> Iterator for ViewRLEDecoder<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter == 0 {
            let byte = self.input.next()?;
            if byte == 0 {
                return None;
            }

            let rle_byte = RLEColorByte::from_bits(byte);
            self.current_color = rle_byte.color();
            self.counter = rle_byte.count() as usize;
        }

        if self.counter == 0 {
            return None;
        }

        self.counter -= 1;

        Some(self.current_color)
    }
}

pub struct ViewRLEEncoder<'a> {
    input: Peekable<&'a mut dyn Iterator<Item = u8>>,
}

impl<'a> ViewRLEEncoder<'a> {
    pub fn new(input: &'a mut dyn Iterator<Item = u8>) -> Self {
        ViewRLEEncoder {
            input: input.peekable(),
        }
    }
}

impl Iterator for ViewRLEEncoder<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let color = self.input.next()?;
        let mut count: u8 = 1;

        loop {
            let next_color = self.input.peek();
            if next_color.is_none() || next_color.unwrap() != &color || count == 15 {
                break;
            }
            self.input.next();
            count += 1;
        }
        let rle_byte = RLEColorByte::new().with_count(count).with_color(color);
        return Some(rle_byte.into_bits());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rle_encoding() {
        let input = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let mut iterator = input.iter().cloned();
        let encoder = ViewRLEEncoder::new(&mut iterator);
        let encoded: Vec<u8> = encoder.collect();
        assert_eq!(encoded, vec![0x13, 0x22, 0x34]);
    }

    #[test]
    fn test_rle_decoding() {
        let input = vec![0x13, 0x22, 0x34];
        let mut iterator = input.iter().cloned();
        let decoder = ViewRLEDecoder::new(&mut iterator);
        let decoded: Vec<u8> = decoder.collect();
        assert_eq!(decoded, vec![1, 1, 1, 2, 2, 3, 3, 3, 3]);
    }
}
