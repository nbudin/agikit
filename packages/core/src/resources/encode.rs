use std::{
    fmt::{Debug, Display},
    io::Cursor,
};

use crate::{
    compression::lzw::{CompressionError, agi_lzw_compress},
    data_encoding::WriteHeterogeneousData,
    logic::encode::AssemblyError,
    resources::{
        ResourceType,
        pack::PackingError,
        resource_collection::{AGIV3ResourceVolNumberWithPicFlag, RESOURCE_SIGNATURE},
    },
};

#[derive(Debug)]
pub enum EncodingError {
    InvalidOptions(String),
    UnencodableData(String),
    IoError(std::io::Error),
    AssemblyError(AssemblyError),
    CompressionError(CompressionError),
    PackingError(PackingError),
}

impl From<std::io::Error> for EncodingError {
    fn from(error: std::io::Error) -> Self {
        EncodingError::IoError(error)
    }
}

impl From<AssemblyError> for EncodingError {
    fn from(error: AssemblyError) -> Self {
        EncodingError::AssemblyError(error)
    }
}

impl From<CompressionError> for EncodingError {
    fn from(error: CompressionError) -> Self {
        EncodingError::CompressionError(error)
    }
}

impl From<PackingError> for EncodingError {
    fn from(error: PackingError) -> Self {
        EncodingError::PackingError(error)
    }
}

impl Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodingError::InvalidOptions(msg) => write!(f, "Invalid options: {}", msg),
            EncodingError::UnencodableData(msg) => write!(f, "Unencodable data: {}", msg),
            EncodingError::IoError(err) => Display::fmt(err, f),
            EncodingError::AssemblyError(err) => Display::fmt(err, f),
            EncodingError::CompressionError(err) => Display::fmt(err, f),
            EncodingError::PackingError(err) => Display::fmt(err, f),
        }
    }
}

pub trait Encode<'opt> {
    type Options: 'opt;

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        out: Out,
        options: Self::Options,
    ) -> Result<(), EncodingError>;
    fn encode_to_vec(&self, options: Self::Options) -> Result<Vec<u8>, EncodingError> {
        let mut data = Vec::new();
        let cursor = Cursor::new(&mut data);
        self.encode(cursor, options)?;
        Ok(data)
    }
}

pub trait EncodeResource<'encopt>: Encode<'encopt> {
    fn resource_type(&self) -> ResourceType;

    fn encode_v2_resource<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        volume_number: u8,
        options: <Self as Encode<'encopt>>::Options,
    ) -> Result<(), EncodingError> {
        let data = self.encode_to_vec(options)?;

        out.write_u16_be(RESOURCE_SIGNATURE)?;
        out.write_u8(volume_number)?;
        out.write_u16_le(data.len() as u16)?;
        out.write(&data)?;

        Ok(())
    }

    fn encode_v3_resource<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        volume_number: u8,
        options: <Self as Encode<'encopt>>::Options,
    ) -> Result<bool, EncodingError> {
        let data = self.encode_to_vec(options)?;
        let data_len = data.len();
        let (data_to_store, compressed_len, used_compression) =
            if self.resource_type() != ResourceType::PIC {
                let compressed_data = agi_lzw_compress(&data)?;
                let compressed_len = compressed_data.len();
                if compressed_len < data_len {
                    (compressed_data, compressed_len, true)
                } else {
                    (data, data_len, false)
                }
            } else {
                (data, data_len, false)
            };

        out.write_u16_be(RESOURCE_SIGNATURE)?;
        out.write_u8(
            AGIV3ResourceVolNumberWithPicFlag::new()
                .with_is_compressed_pic(self.resource_type() == ResourceType::PIC)
                .with_volume_number(volume_number)
                .into_bits(),
        )?;
        out.write_u16_le(data_len as u16)?;
        out.write_u16_le(compressed_len as u16)?;
        out.write(&data_to_store)?;

        Ok(used_compression)
    }
}

#[cfg(feature = "js")]
mod js {
    use std::io::{Cursor, Write};

    use serde::{Deserialize, Serialize};
    use tsify::Tsify;
    use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

    use crate::{
        logic::LogicProgram,
        picture::Picture,
        resources::{ResourceType, encode::EncodeResource, pack::EncodedResource},
        sound::ibm_pcjr::sound::IBMPCjrSound,
        views::AGIView,
    };

    #[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
    #[tsify(into_wasm_abi, from_wasm_abi)]
    pub enum JsResource {
        Logic(LogicProgram),
        Picture(Picture),
        IBMPCJrSound(IBMPCjrSound),
        View(AGIView),
    }

    impl JsResource {
        pub fn resource_type(&self) -> ResourceType {
            match self {
                JsResource::Logic(_) => ResourceType::LOGIC,
                JsResource::Picture(_) => ResourceType::PIC,
                JsResource::IBMPCJrSound(_) => ResourceType::SOUND,
                JsResource::View(_) => ResourceType::VIEW,
            }
        }
    }

    #[wasm_bindgen(js_name = "encodeV2Resource")]
    pub fn js_encode_v2_resource(
        volume_number: u8,
        resource_number: u8,
        resource: JsResource,
    ) -> Result<EncodedResource, JsValue> {
        let mut buf: Vec<u8> = vec![];
        let out = Cursor::new(&mut buf);
        let resource_type = match &resource {
            JsResource::Logic(logic_program) => {
                logic_program.encode_v2_resource(out, volume_number, true)
            }
            JsResource::Picture(picture) => picture.encode_v2_resource(out, volume_number, false),
            JsResource::IBMPCJrSound(ibmpcjr_sound) => {
                ibmpcjr_sound.encode_v2_resource(out, volume_number, ())
            }
            JsResource::View(view) => view.encode_v2_resource(out, volume_number, ()),
        }
        .map(|_| resource.resource_type())
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

        Ok(EncodedResource {
            resource_type: resource_type,
            resource_number,
            data: buf,
        })
    }

    #[wasm_bindgen(js_name = "encodeV3Resource")]
    pub fn js_encode_v3_resource(
        volume_number: u8,
        resource_number: u8,
        resource: JsResource,
    ) -> Result<EncodedResource, JsValue> {
        let mut buf: Vec<u8> = vec![];
        let mut out = Cursor::new(&mut buf);
        let resource_type = match &resource {
            JsResource::Logic(logic_program) => {
                let mut temp_buf: Vec<u8> = vec![];
                let temp_out = Cursor::new(&mut temp_buf);

                // First try not encrypting and see if compression saves space
                logic_program
                    .encode_v3_resource(temp_out, volume_number, false)
                    .and_then(|used_compression| {
                        if used_compression {
                            out.write(&temp_buf)
                                .map(|_| used_compression)
                                .map_err(|err| err.into())
                        } else {
                            // If it doesn't, encrypt the messages
                            logic_program.encode_v3_resource(out, volume_number, true)
                        }
                    })
            }
            JsResource::Picture(picture) => picture.encode_v3_resource(out, volume_number, true),
            JsResource::IBMPCJrSound(ibmpcjr_sound) => {
                ibmpcjr_sound.encode_v3_resource(out, volume_number, ())
            }
            JsResource::View(view) => view.encode_v3_resource(out, volume_number, ()),
        }
        .map(|_| resource.resource_type())
        .map_err(|err| JsValue::from_str(format!("{}", err).as_str()))?;

        Ok(EncodedResource {
            resource_type: resource_type,
            resource_number,
            data: buf,
        })
    }
}
