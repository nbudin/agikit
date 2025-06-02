use std::collections::HashMap;

use crate::{
    resource::{Encode, EncodingError},
    views::cel::ViewCelPixelsIterator,
};

use super::{
    cel::{NonMirroredViewCelData, TransparencyMirroringByte, ViewCel, ViewCelData},
    rle::ViewRLEEncoder,
    AGIView, ViewLoop,
};

impl Encode for ViewCel {
    type Options = Option<u8>; // Mirrored from loop number
    fn encode(&self, mirrored_from_loop_number: Option<u8>) -> Result<Vec<u8>, EncodingError> {
        let ViewCelData::NonMirrored(NonMirroredViewCelData { data }) = &self.data else {
            return Err(EncodingError::InvalidOptions(
                "Only non-mirrored cel data can be encoded".to_string(),
            ));
        };

        let mut encoded = Vec::new();
        encoded.push(self.width);
        encoded.push(self.height);

        let transparency_mirroring_byte = TransparencyMirroringByte::new()
            .with_transparent_color(self.transparent_color)
            .with_is_mirrored(mirrored_from_loop_number.is_some())
            .with_mirrored_from_loop_number(mirrored_from_loop_number.unwrap_or(0));
        encoded.push(transparency_mirroring_byte.into_bits());

        let mut data_iterator = data.iter().copied();
        let encoder = ViewRLEEncoder::new(
            &mut data_iterator,
            self.width as usize,
            self.height as usize,
            self.transparent_color,
        );
        let encoded_data = encoder.collect::<Vec<u8>>();

        let mut mirrored_iterator =
            ViewCelPixelsIterator::new(&data, true, self.width, self.height);
        let mirrored_encoder = ViewRLEEncoder::new(
            &mut mirrored_iterator,
            self.width as usize,
            self.height as usize,
            self.transparent_color,
        );
        let mirrored_count = mirrored_encoder.count();
        let target_byte_count = encoded_data.len().max(mirrored_count);
        let pad_bytes = target_byte_count.saturating_sub(encoded_data.len());

        encoded.extend(encoded_data.iter().copied());
        if pad_bytes > 0 {
            encoded.extend(std::iter::repeat(0).take(pad_bytes));
        }

        eprintln!(
            "Cel {} encoded with length {}",
            self.cel_number,
            encoded.len()
        );

        Ok(encoded)
    }
}

impl Encode for ViewLoop {
    type Options = Option<u8>; // Mirrored from loop number

    fn encode(&self, mirrored_from_loop_number: Option<u8>) -> Result<Vec<u8>, EncodingError> {
        let mut cels_encoded = Vec::new();
        for cel in &self.cels {
            cels_encoded.push(cel.encode(mirrored_from_loop_number)?);
        }

        let cel_offsets = cels_encoded
            .iter()
            .fold(
                (1 + self.cels.len() * 2, vec![]),
                |(offset, mut acc), cel| {
                    acc.push(offset as u16);
                    (offset + cel.len(), acc)
                },
            )
            .1;

        eprintln!("Loop {} Cel offsets: {:?}", self.loop_number, cel_offsets);

        Ok(std::iter::once(self.cels.len() as u8)
            .chain(cel_offsets.iter().flat_map(|&offset| offset.to_le_bytes()))
            .chain(cels_encoded.iter().flatten().copied())
            .collect())
    }
}

impl Encode for AGIView {
    type Options = ();
    fn encode(&self, _: ()) -> Result<Vec<u8>, EncodingError> {
        let mut encoded = Vec::new();

        let mirror_source_loop_numbers: HashMap<u8, u8> = self
            .loops
            .iter()
            .flat_map(|loop_| {
                loop_.cels.iter().filter_map(|cel| {
                    if cel.is_mirrored() {
                        Some((loop_.loop_number, cel.mirrored_from_loop_number().unwrap()))
                    } else {
                        None
                    }
                })
            })
            .collect();
        let mirror_destination_loop_numbers = mirror_source_loop_numbers
            .iter()
            .map(|(k, v)| (*v, *k))
            .collect::<HashMap<_, _>>();

        let encoded_loops: HashMap<u8, Vec<u8>> = self
            .loops
            .iter()
            .filter_map(|loop_| {
                if mirror_source_loop_numbers.get(&loop_.loop_number).is_some() {
                    return None;
                }

                let loop_number_if_mirrored = mirror_destination_loop_numbers
                    .get(&loop_.loop_number)
                    .map(|_| loop_.loop_number);

                Some(
                    loop_
                        .encode(loop_number_if_mirrored)
                        .map(|encoded| (loop_.loop_number, encoded)),
                )
            })
            .collect::<Result<_, _>>()?;

        let loop_header_length = 2 // two unknown header bytes
         + 1 // number of loops
          + 2 // description offset
           + self.loops.len() * 2;

        let non_mirrored_loop_offsets = self
            .loops
            .iter()
            .filter_map(|loop_| {
                encoded_loops
                    .get(&loop_.loop_number)
                    .map(|encoded| (loop_.loop_number, encoded))
            })
            .fold(
                (
                    loop_header_length + self.description.as_ref().map_or(0, |desc| desc.len() + 1),
                    HashMap::new(),
                ),
                |(offset, mut acc), (loop_number, encoded)| {
                    acc.insert(loop_number, offset);
                    (offset + encoded.len(), acc)
                },
            )
            .1;

        let loop_offsets = self
            .loops
            .iter()
            .map(|loop_| {
                let source_loop_number = mirror_source_loop_numbers
                    .get(&loop_.loop_number)
                    .copied()
                    .unwrap_or(loop_.loop_number);

                non_mirrored_loop_offsets
                    .get(&source_loop_number)
                    .copied()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        encoded.extend(
            [1, 1, self.loops.len() as u8]
                .iter()
                .copied()
                .chain(
                    self.description
                        .as_ref()
                        .map_or_else(|| [0, 0], |_| (loop_header_length as u16).to_le_bytes()),
                )
                .chain(
                    loop_offsets
                        .iter()
                        .flat_map(|offset| (*offset as u16).to_le_bytes()),
                )
                .collect::<Vec<u8>>(),
        );

        if let Some(description) = &self.description {
            encoded.extend(description.as_bytes());
            encoded.push(0); // Null terminator
        }

        for loop_ in self.loops.iter() {
            if let Some(loop_encoded) = encoded_loops.get(&loop_.loop_number) {
                encoded.extend(loop_encoded);
            }
        }

        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn build_test_cel() -> ViewCel {
        ViewCel {
            cel_number: 0,
            width: 2,
            height: 2,
            transparent_color: 15,
            data: ViewCelData::NonMirrored(NonMirroredViewCelData {
                data: vec![1, 2, 3, 4],
            }),
        }
    }

    fn build_test_loop(cels: Vec<ViewCel>) -> ViewLoop {
        ViewLoop {
            loop_number: 0,
            cels,
        }
    }

    #[test]
    fn test_encode_cel() {
        let cel = build_test_cel();
        let encoded = cel.encode(None).unwrap();
        assert_eq!(
            encoded,
            vec![
                2,    // width
                2,    // height
                0x0f, // transparency mirroring byte (transparent color 0, not mirrored)
                0x11, 0x21, 0x31, 0x41, 0 // RLE encoded data for [1, 2, 3, 4]
            ]
        );
    }

    #[test]
    fn test_encode_loop() {
        let cel = build_test_cel();
        let cel_encoded = cel.encode(None).unwrap();
        let loop_: ViewLoop = build_test_loop(vec![cel]);
        let loop_encoded = loop_.encode(None).unwrap();
        assert_eq!(
            loop_encoded,
            vec![
                vec![
                    1, // cel count
                    3, 0, // cel 0 offset
                ],
                cel_encoded
            ]
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<u8>>()
        );
    }

    #[test]
    fn test_encode_view() {
        let cel = build_test_cel();
        let loop_: ViewLoop = build_test_loop(vec![cel]);
        let loop_encoded = loop_.encode(None).unwrap();
        let view = AGIView {
            loops: vec![loop_],
            description: Some("Test View".to_string()),
        };
        let view_encoded = view.encode(()).unwrap();

        assert_eq!(
            view_encoded,
            vec![
                vec![
                    1, // unknown byte
                    1, // unknown byte
                    1, // number of loops
                    7,
                    0, // description offset (0)
                    (7 + "Test View".len() + 1) as u8,
                    0, // loop 0 offset
                ],
                "Test View\0".as_bytes().to_vec(), // description
                loop_encoded
            ]
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<u8>>()
        );
    }
}
