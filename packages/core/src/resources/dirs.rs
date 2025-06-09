use std::collections::HashMap;

use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    data_encoding::ReadHeterogeneousData,
    resources::{
        decode::{Decode, DecodingError},
        ResourceType,
    },
};

#[wasm_bindgen(skip_typescript)]
pub struct DirEntry {
    #[wasm_bindgen(skip)]
    pub resource_type: ResourceType,
    #[wasm_bindgen(js_name = "resourceNumber")]
    pub resource_number: u8,
    #[wasm_bindgen(js_name = "volumeNumber")]
    pub volume_number: u8,
    pub offset: u32,
}

#[wasm_bindgen]
impl DirEntry {
    #[wasm_bindgen(getter, js_name = "resourceType", skip_typescript)]
    pub fn js_resource_type(&self) -> String {
        self.resource_type.as_ref().to_string()
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export class DirEntry {
  private constructor();
  free(): void;
  resourceNumber: number;
  volumeNumber: number;
  offset: number;
  readonly resourceType: ResourceType;
}
"#;

impl<Data: ReadHeterogeneousData> Decode<'_, Data> for Option<DirEntry> {
    type Options = (ResourceType, u8);

    fn decode<'a>(
        data: &'a mut Data,
        (resource_type, resource_number): Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let vol_plus_high_order_nybble = data.read_u8()?;
        let mid_order_byte = data.read_u8()?;
        let low_order_byte = data.read_u8()?;

        let volume_number = vol_plus_high_order_nybble >> 4;
        let offset = ((vol_plus_high_order_nybble & 0x0F) as u32) << 16
            | ((mid_order_byte as u32) << 8)
            | low_order_byte as u32;

        if offset == 0xfffff && volume_number == 0x0f {
            Ok(None)
        } else {
            Ok(Some(DirEntry {
                resource_type,
                resource_number,
                volume_number,
                offset,
            }))
        }
    }
}

impl<Data: ReadHeterogeneousData> Decode<'_, Data> for HashMap<u8, DirEntry> {
    type Options = ResourceType;

    fn decode<'a>(data: &'a mut Data, resource_type: Self::Options) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut entries = HashMap::new();
        let mut resource_number: u8 = 0;

        loop {
            let decode_result = Option::<DirEntry>::decode(data, (resource_type, resource_number));

            match decode_result {
                Ok(Some(dir_entry)) => {
                    entries.insert(resource_number, dir_entry);
                }
                Ok(None) => {}
                Err(err) => match err {
                    DecodingError::IoError(err) => {
                        if err.kind() == std::io::ErrorKind::UnexpectedEof {
                            break; // End of data reached
                        } else {
                            return Err(DecodingError::IoError(err));
                        }
                    }
                },
            }

            resource_number += 1;
        }

        Ok(entries)
    }
}
