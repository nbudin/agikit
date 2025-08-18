pub mod decode;
pub mod encode;
pub mod sound;

#[cfg(test)]
mod tests {
    use similar_asserts::assert_eq;

    use crate::{
        resources::{ResourceType, decode::Decode, encode::Encode},
        sound::ibm_pcjr::sound::IBMPCjrSound,
        test_data::uriquest,
    };

    #[test]
    fn smoke_test() {
        let sound_data = uriquest()
            .read_resource_data(ResourceType::SOUND, 19)
            .unwrap();
        let sound = IBMPCjrSound::decode_from_bytes(&sound_data.data, ()).unwrap();
        let reencoded = sound.encode_to_vec(()).unwrap();
        assert_eq!(sound_data.data[0..8], reencoded[0..8]);
    }
}
