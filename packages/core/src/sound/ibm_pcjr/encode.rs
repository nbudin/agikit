use crate::{
    data_encoding::WriteHeterogeneousData,
    resources::{
        ResourceType,
        encode::{Encode, EncodeResource, EncodingError},
    },
    sound::{
        SoundNote,
        ibm_pcjr::sound::{
            IBMPCjrNoiseNote, IBMPCjrNoiseVoice, IBMPCjrSound, IBMPCjrToneNote, IBMPCjrToneVoice,
        },
    },
};

struct InsertSilentNotesIterator<'a, NoteType: SoundNote, TSF: Fn(NoteType) -> NoteType> {
    notes: &'a mut dyn Iterator<Item = NoteType>,
    current_time: u32,
    next_note: Option<NoteType>,
    transform_silent: Box<TSF>,
}

impl<'a, NoteType: SoundNote, TSF: Fn(NoteType) -> NoteType>
    InsertSilentNotesIterator<'a, NoteType, TSF>
{
    pub fn new<I: Iterator<Item = NoteType>>(notes: &'a mut I, transform_silent: TSF) -> Self {
        Self {
            notes: notes as &'a mut dyn Iterator<Item = NoteType>,
            current_time: 0,
            next_note: None,
            transform_silent: Box::new(transform_silent),
        }
    }
}

impl<'a, NoteType: SoundNote + Clone, TSF: Fn(NoteType) -> NoteType> Iterator
    for InsertSilentNotesIterator<'a, NoteType, TSF>
{
    type Item = NoteType;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_note) = self.next_note.take() {
            self.current_time += next_note.duration();
            return Some(next_note);
        }

        let Some(note) = self.notes.next() else {
            return None;
        };

        if note.start_time() > self.current_time {
            self.next_note = Some(note.clone());
            let silent_note = (self.transform_silent)(NoteType::silent(
                self.current_time,
                note.start_time() - self.current_time,
            ));
            self.current_time += silent_note.duration();
            return Some(silent_note);
        }

        self.current_time += note.duration();
        Some(note)
    }
}

impl Encode<'_> for IBMPCjrToneNote {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        out.write_u16_le(self.duration as u16)?;
        out.write_u16_be(self.frequency_divisor.into_bits())?;
        out.write_u8(self.attenuation_byte.into_bits())?;
        Ok(())
    }
}

impl Encode<'_> for IBMPCjrNoiseNote {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        out.write_u16_le(self.duration as u16)?;
        out.write_u8(0)?;
        out.write_u8(self.frequency_byte.into_bits())?;
        out.write_u8(self.attenuation_byte.into_bits())?;
        Ok(())
    }
}

impl Encode<'_> for IBMPCjrToneVoice {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let mut notes_iter = self.notes.iter().cloned();
        let insert_silent =
            InsertSilentNotesIterator::new(&mut notes_iter, |silent_note| IBMPCjrToneNote {
                start_time: silent_note.start_time,
                duration: silent_note.duration,
                frequency_divisor: silent_note
                    .frequency_divisor
                    .with_t1_register(self.channel.voice_register()),
                attenuation_byte: silent_note
                    .attenuation_byte
                    .with_t1_register(self.channel.attenuation_register()),
            });

        for note in insert_silent {
            note.encode(&mut out, ())?;
        }

        out.write_u16_le(0xffff)?;

        Ok(())
    }
}

impl Encode<'_> for IBMPCjrNoiseVoice {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let mut notes_iter = self.notes.iter().cloned();
        let insert_silent =
            InsertSilentNotesIterator::new(&mut notes_iter, |silent_note| IBMPCjrNoiseNote {
                start_time: silent_note.start_time,
                duration: silent_note.duration,
                frequency_byte: silent_note.frequency_byte.with_t1_register(0b110),
                attenuation_byte: silent_note.attenuation_byte.with_t1_register(0b111),
            });

        for note in insert_silent {
            note.encode(&mut out, ())?;
        }

        out.write_u16_le(0xffff)?;

        Ok(())
    }
}

impl Encode<'_> for IBMPCjrSound {
    type Options = ();

    fn encode<Out: WriteHeterogeneousData>(
        &self,
        mut out: Out,
        _options: Self::Options,
    ) -> Result<(), EncodingError> {
        let tone_data = self
            .tone_voices
            .iter()
            .map(|voice| voice.encode_to_vec(()))
            .collect::<Result<Vec<_>, _>>()?;

        let noise_data = self.noise_voice.encode_to_vec(())?;

        let mut offset: u16 = 8;
        for tone_voice_data in tone_data.iter() {
            out.write_u16_le(offset)?;
            offset += tone_voice_data.len() as u16;
        }
        out.write_u16_le(offset)?;

        for tone_voice_data in tone_data.iter() {
            out.write(&tone_voice_data)?;
        }
        out.write(&noise_data)?;

        Ok(())
    }
}

impl EncodeResource<'_> for IBMPCjrSound {
    fn resource_type(&self) -> ResourceType {
        ResourceType::SOUND
    }
}
