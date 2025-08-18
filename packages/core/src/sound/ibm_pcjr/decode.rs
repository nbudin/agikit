use std::io::SeekFrom;

use crate::{
    data_encoding::ReadHeterogeneousData,
    resources::decode::{Decode, DecodingError},
    sound::ibm_pcjr::sound::{
        IBMPCjrAttenuationByte, IBMPCjrFrequencyDivisor, IBMPCjrNoiseFrequencyByte,
        IBMPCjrNoiseNote, IBMPCjrNoiseVoice, IBMPCjrSound, IBMPCjrToneChannel, IBMPCjrToneNote,
        IBMPCjrToneVoice,
    },
};

impl Decode<'_> for IBMPCjrToneVoice {
    type Options = (IBMPCjrToneChannel, u64); // channel, end offset

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        (channel, end_offset): Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut notes: Vec<IBMPCjrToneNote> = vec![];
        let mut start_time: u32 = 0;

        while data.stream_position()? < end_offset {
            let duration = data.read_u16_le()? as u32;
            if duration == 0xffff {
                break;
            }

            let frequency_divisor = IBMPCjrFrequencyDivisor::from_bits(data.read_u16_be()?);
            let attenuation_byte = IBMPCjrAttenuationByte::from_bits(data.read_u8()?);

            notes.push(IBMPCjrToneNote {
                start_time,
                duration,
                frequency_divisor,
                attenuation_byte,
            });

            start_time += duration;
        }

        Ok(IBMPCjrToneVoice { notes, channel })
    }
}

impl Decode<'_> for IBMPCjrNoiseVoice {
    type Options = u64; // end offset

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        end_offset: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        let mut notes: Vec<IBMPCjrNoiseNote> = vec![];
        let mut start_time: u32 = 0;

        while data.stream_position()? < end_offset {
            let duration = data.read_u16_le()? as u32;
            if duration == 0xffff {
                break;
            }

            // third byte is always zero for the noise voice
            data.seek_relative(1)?;
            let frequency_byte = IBMPCjrNoiseFrequencyByte::from_bits(data.read_u8()?);
            let attenuation_byte = IBMPCjrAttenuationByte::from_bits(data.read_u8()?);

            notes.push(IBMPCjrNoiseNote {
                start_time,
                duration,
                frequency_byte,
                attenuation_byte,
            });

            start_time += duration;
        }

        Ok(IBMPCjrNoiseVoice { notes })
    }
}

impl Decode<'_> for IBMPCjrSound {
    type Options = ();

    fn decode<'a, Data: ReadHeterogeneousData>(
        data: &'a mut Data,
        _options: Self::Options,
    ) -> Result<Self, DecodingError>
    where
        Self: Sized,
    {
        data.seek(SeekFrom::End(0))?;
        let data_length = data.stream_position()?;
        data.seek(SeekFrom::Start(0))?;

        let tone_voice_offsets = (0..3)
            .map(|_| data.read_u16_le())
            .collect::<Result<Vec<_>, _>>()?;
        let noise_voice_offset = data.read_u16_le()?;

        data.seek(SeekFrom::Start(tone_voice_offsets[0].into()))?;
        let tone1 = IBMPCjrToneVoice::decode(
            data,
            (IBMPCjrToneChannel::Tone1, tone_voice_offsets[1].into()),
        )?;
        data.seek(SeekFrom::Start(tone_voice_offsets[1].into()))?;
        let tone2 = IBMPCjrToneVoice::decode(
            data,
            (IBMPCjrToneChannel::Tone2, tone_voice_offsets[2].into()),
        )?;
        data.seek(SeekFrom::Start(tone_voice_offsets[2].into()))?;
        let tone3 =
            IBMPCjrToneVoice::decode(data, (IBMPCjrToneChannel::Tone3, noise_voice_offset.into()))?;
        data.seek(SeekFrom::Start(noise_voice_offset.into()))?;
        let noise_voice = IBMPCjrNoiseVoice::decode(data, data_length)?;

        Ok(IBMPCjrSound {
            tone_voices: [tone1, tone2, tone3],
            noise_voice,
        })
    }
}
