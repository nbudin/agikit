use std::iter::Peekable;

use bitfield_struct::bitfield;

#[bitfield(u8)]
struct RLEColorByte {
    #[bits(4)]
    count: u8,
    #[bits(4)]
    color: u8,
}

#[derive(Debug)]
enum ViewRLEDecoderState {
    Start,
    OutputColor { color: u8, count: usize },
    FillTransparent { count: usize },
    ReachedLineEnd,
    Done,
}

pub struct ViewRLEDecoder<'a> {
    input: &'a mut dyn Iterator<Item = u8>,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    transparent_color: u8,
    state: ViewRLEDecoderState,
}

impl<'a> ViewRLEDecoder<'a> {
    pub fn new(
        input: &'a mut dyn Iterator<Item = u8>,
        width: usize,
        height: usize,
        transparent_color: u8,
    ) -> Self {
        ViewRLEDecoder {
            input,
            transparent_color,
            x: 0,
            y: 0,
            width,
            height,
            state: ViewRLEDecoderState::Start,
        }
    }

    fn finish_line(&mut self) {
        self.x = 0;
        self.y += 1;
        if self.y >= self.height {
            self.state = ViewRLEDecoderState::Done;
        } else {
            self.state = ViewRLEDecoderState::Start;
        }
    }
}

impl<'a> Iterator for ViewRLEDecoder<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            ViewRLEDecoderState::Start => {
                let byte = self.input.next()?;
                if byte > 0 {
                    let byte = RLEColorByte::from_bits(byte);
                    self.state = ViewRLEDecoderState::OutputColor {
                        color: byte.color(),
                        count: byte.count() as usize,
                    };
                } else {
                    self.state = ViewRLEDecoderState::ReachedLineEnd;
                }
                self.next()
            }
            ViewRLEDecoderState::OutputColor { color, count } => {
                let new_count = count - 1;
                if new_count == 0 {
                    if self.x >= self.width {
                        self.state = ViewRLEDecoderState::ReachedLineEnd;
                    } else {
                        self.state = ViewRLEDecoderState::Start;
                    }
                } else {
                    self.state = ViewRLEDecoderState::OutputColor {
                        color,
                        count: new_count,
                    };
                }
                self.x += 1;
                Some(color)
            }
            ViewRLEDecoderState::FillTransparent { count } => {
                let new_count = count - 1;
                if new_count == 0 {
                    self.state = ViewRLEDecoderState::ReachedLineEnd;
                } else {
                    self.state = ViewRLEDecoderState::FillTransparent { count: new_count };
                }
                self.x += 1;
                Some(self.transparent_color)
            }
            ViewRLEDecoderState::ReachedLineEnd => {
                if self.x < self.width {
                    let fill_count = self.width - self.x;
                    self.state = ViewRLEDecoderState::FillTransparent { count: fill_count };
                } else {
                    self.finish_line();
                }
                self.next()
            }
            ViewRLEDecoderState::Done => None,
        }
    }
}

#[derive(Debug)]
enum ViewRLEEncoderState {
    Start,
    ReachedLineEnd,
    Done,
}

pub struct ViewRLEEncoder<'a, Data: Iterator<Item = u8> + 'a> {
    input: Peekable<&'a mut Data>,
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    transparent_color: u8,
    state: ViewRLEEncoderState,
}

impl<'a, Data: Iterator<Item = u8> + 'a> ViewRLEEncoder<'a, Data> {
    pub fn new(input: &'a mut Data, width: usize, height: usize, transparent_color: u8) -> Self {
        ViewRLEEncoder {
            input: input.peekable(),
            width,
            height,
            transparent_color,
            x: 0,
            y: 0,
            state: ViewRLEEncoderState::Start,
        }
    }

    fn advance(&mut self) {
        self.x += 1;
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
    }
}

impl<Data: Iterator<Item = u8>> Iterator for ViewRLEEncoder<'_, Data> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        eprintln!(
            "RLE Encoder: x={}, y={}, state={:?}",
            self.x, self.y, self.state
        );
        match self.state {
            ViewRLEEncoderState::Start => {
                let Some(color) = self.input.next() else {
                    self.state = ViewRLEEncoderState::Done;
                    return Some(0);
                };

                self.advance();

                let mut count: u8 = 1;

                loop {
                    let next_color = self.input.peek();
                    if next_color.is_none()
                        || next_color.unwrap() != &color
                        || count == 15
                        || self.x == 0
                    {
                        break;
                    }
                    self.advance();
                    self.input.next();
                    count += 1;
                }

                eprintln!("RLE Encoder: color={}, count={}", color, count);

                if self.x == 0 {
                    self.state = ViewRLEEncoderState::ReachedLineEnd;

                    if color == self.transparent_color {
                        // Skip returning the RLE data for transparent colors
                        return self.next();
                    }
                }

                let rle_byte = RLEColorByte::new().with_count(count).with_color(color);
                return Some(rle_byte.into_bits());
            }
            ViewRLEEncoderState::ReachedLineEnd => {
                self.state = if self.y < self.height {
                    ViewRLEEncoderState::Start
                } else {
                    ViewRLEEncoderState::Done
                };
                Some(0)
            }
            ViewRLEEncoderState::Done => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rle_encoding() {
        let input = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let mut iterator = input.iter().cloned();
        let encoder = ViewRLEEncoder::new(&mut iterator, 3, 3, 3);
        let encoded: Vec<u8> = encoder.collect();
        assert_eq!(encoded, vec![0x13, 0, 0x22, 0, 0]);
    }

    #[test]
    fn test_rle_decoding() {
        let input = vec![0x13, 0, 0x22, 0, 0];
        let mut iterator = input.iter().cloned();
        let decoder = ViewRLEDecoder::new(&mut iterator, 3, 3, 3);
        let decoded: Vec<u8> = decoder.collect();
        assert_eq!(decoded, vec![1, 1, 1, 2, 2, 3, 3, 3, 3]);
    }
}
