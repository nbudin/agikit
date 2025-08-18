use bitfield_struct::bitfield;
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::sound::SoundNote;

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(rename_all = "camelCase")]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum IBMPCjrToneChannel {
    Tone1,
    Tone2,
    Tone3,
}

impl IBMPCjrToneChannel {
    pub fn voice_register(&self) -> u8 {
        match self {
            IBMPCjrToneChannel::Tone1 => 0b000,
            IBMPCjrToneChannel::Tone2 => 0b010,
            IBMPCjrToneChannel::Tone3 => 0b100,
        }
    }

    pub fn attenuation_register(&self) -> u8 {
        match self {
            IBMPCjrToneChannel::Tone1 => 0b001,
            IBMPCjrToneChannel::Tone2 => 0b011,
            IBMPCjrToneChannel::Tone3 => 0b101,
        }
    }
}

#[bitfield(u16)]
#[derive(Serialize, Deserialize)]
pub struct IBMPCjrFrequencyDivisor {
    #[bits(6)]
    divisor1: u8,
    #[serde(skip)]
    _unused1: bool,
    #[bits(default = false)]
    always_0: bool,
    #[bits(4)]
    divisor2: u8,
    #[bits(3)]
    pub t1_register: u8,
    #[bits(default = true)]
    always_1: bool,
}

impl IBMPCjrFrequencyDivisor {
    pub fn divisor(&self) -> u16 {
        ((self.divisor1() as u16) << 4) + self.divisor2() as u16
    }

    pub fn set_divisor(&mut self, divisor: u16) {
        self.set_divisor1((divisor >> 4) as u8);
        self.set_divisor2((divisor & 0x0f) as u8);
    }

    pub fn with_divisor(&self, divisor: u16) -> Self {
        let mut cloned = self.clone();
        cloned.set_divisor(divisor);
        cloned
    }

    pub fn frequency(&self) -> f64 {
        (111860.0 / self.divisor() as f64).floor()
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.set_divisor((111860.0 / frequency).floor() as u16);
    }

    pub fn with_frequency(&self, frequency: f64) -> Self {
        let mut cloned = self.clone();
        cloned.set_frequency(frequency);
        cloned
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Attenuation {
    Decibels(u8),
    VolumeOff,
}

impl Attenuation {
    const fn into_bits(self) -> u8 {
        match self {
            Attenuation::Decibels(db) => db / 2,
            Attenuation::VolumeOff => 15,
        }
    }

    const fn from_bits(value: u8) -> Attenuation {
        match value {
            15 => Attenuation::VolumeOff,
            _ => Attenuation::Decibels(value * 2),
        }
    }
}

#[bitfield(u8)]
#[derive(Serialize, Deserialize)]
pub struct IBMPCjrAttenuationByte {
    #[bits(4)]
    pub attenuation_value: Attenuation,
    #[bits(3)]
    pub t1_register: u8,
    #[serde(skip)]
    _unused: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct IBMPCjrToneNote {
    #[wasm_bindgen(js_name = "startTime")]
    pub start_time: u32,
    pub duration: u32,
    #[wasm_bindgen(skip)]
    pub frequency_divisor: IBMPCjrFrequencyDivisor,
    #[wasm_bindgen(skip)]
    pub attenuation_byte: IBMPCjrAttenuationByte,
}

impl SoundNote for IBMPCjrToneNote {
    fn silent(start_time: u32, duration: u32) -> Self {
        IBMPCjrToneNote {
            start_time,
            duration,
            frequency_divisor: IBMPCjrFrequencyDivisor::new()
                .with_always_0(false)
                .with_always_1(true),
            attenuation_byte: IBMPCjrAttenuationByte::new()
                .with_attenuation_value(Attenuation::VolumeOff),
        }
    }

    fn duration(&self) -> u32 {
        self.duration
    }

    fn start_time(&self) -> u32 {
        self.start_time
    }

    fn is_silent(&self) -> bool {
        self.attenuation_byte.attenuation_value() == Attenuation::VolumeOff
    }
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl IBMPCjrToneNote {
    #[wasm_bindgen(js_name = "frequency", getter)]
    pub fn js_frequency(&self) -> f64 {
        self.frequency_divisor.frequency()
    }

    #[wasm_bindgen(js_name = "attenuation", getter)]
    pub fn js_attenuation(&self) -> u8 {
        match self.attenuation_byte.attenuation_value() {
            Attenuation::Decibels(db) => db,
            Attenuation::VolumeOff => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(from_wasm_abi, into_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[repr(u8)]
pub enum IBMPCjrNoiseType {
    White = 1,
    Periodic = 0,
}

impl IBMPCjrNoiseType {
    const fn into_bits(self) -> u8 {
        self as _
    }
    const fn from_bits(value: u8) -> IBMPCjrNoiseType {
        match value {
            1 => IBMPCjrNoiseType::White,
            _ => IBMPCjrNoiseType::Periodic,
        }
    }
}

#[bitfield(u8)]
#[derive(Serialize, Deserialize)]
pub struct IBMPCjrNoiseFrequencyByte {
    #[bits(2)]
    frequency_divisor_log_minus_9: u8,
    #[bits(1)]
    pub noise_type: IBMPCjrNoiseType,
    #[serde(skip)]
    _unused1: bool,
    #[bits(3)]
    pub t1_register: u8,
    #[bits(default = true)]
    always_1: bool,
}

impl IBMPCjrNoiseFrequencyByte {
    pub fn frequency(&self) -> f64 {
        (1193180.0_f64 / 2.0).powi(self.frequency_divisor_log_minus_9() as i32 + 9)
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        let frequency_divisor = (1193180.0 / frequency).floor();
        self.set_frequency_divisor_log_minus_9((frequency_divisor.log2().floor() - 9.0) as u8);
    }

    pub fn with_frequency(&self, frequency: f64) -> Self {
        let mut cloned = self.clone();
        cloned.set_frequency(frequency);
        cloned
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct IBMPCjrNoiseNote {
    #[wasm_bindgen(js_name = "startTime")]
    pub start_time: u32,
    pub duration: u32,
    #[wasm_bindgen(skip)]
    pub frequency_byte: IBMPCjrNoiseFrequencyByte,
    #[wasm_bindgen(skip)]
    pub attenuation_byte: IBMPCjrAttenuationByte,
}

impl SoundNote for IBMPCjrNoiseNote {
    fn silent(start_time: u32, duration: u32) -> Self {
        IBMPCjrNoiseNote {
            start_time,
            duration,
            frequency_byte: IBMPCjrNoiseFrequencyByte::new()
                .with_t1_register(0b110)
                .with_noise_type(IBMPCjrNoiseType::White)
                .with_frequency_divisor_log_minus_9(0),
            attenuation_byte: IBMPCjrAttenuationByte::new()
                .with_attenuation_value(Attenuation::VolumeOff)
                .with_t1_register(0b111),
        }
    }

    fn is_silent(&self) -> bool {
        self.attenuation_byte.attenuation_value() == Attenuation::VolumeOff
    }

    fn duration(&self) -> u32 {
        self.duration
    }

    fn start_time(&self) -> u32 {
        self.start_time
    }
}

#[wasm_bindgen]
impl IBMPCjrNoiseNote {
    #[wasm_bindgen(js_name = "frequency")]
    pub fn js_frequency(&self) -> f64 {
        self.frequency_byte.frequency()
    }

    #[wasm_bindgen(js_name = "attenuation")]
    pub fn attenuation(&self) -> u8 {
        match self.attenuation_byte.attenuation_value() {
            Attenuation::Decibels(db) => db,
            Attenuation::VolumeOff => 15,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct IBMPCjrToneVoice {
    #[wasm_bindgen(getter_with_clone)]
    pub channel: IBMPCjrToneChannel,
    #[wasm_bindgen(skip)]
    pub notes: Vec<IBMPCjrToneNote>,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl IBMPCjrToneVoice {
    #[wasm_bindgen(getter, js_name = "notes")]
    pub fn js_notes(&self) -> Vec<IBMPCjrToneNote> {
        self.notes
            .iter()
            .filter(|note| !note.is_silent())
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct IBMPCjrNoiseVoice {
    #[wasm_bindgen(skip)]
    pub notes: Vec<IBMPCjrNoiseNote>,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl IBMPCjrNoiseVoice {
    #[wasm_bindgen(getter, js_name = "notes")]
    pub fn js_notes(&self) -> Vec<IBMPCjrNoiseNote> {
        self.notes
            .iter()
            .filter(|note| !note.is_silent())
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct IBMPCjrSound {
    #[wasm_bindgen(skip)]
    pub tone_voices: [IBMPCjrToneVoice; 3],
    #[wasm_bindgen(js_name = "noiseVoice", getter_with_clone)]
    pub noise_voice: IBMPCjrNoiseVoice,
}

#[cfg(feature = "js")]
#[wasm_bindgen]
impl IBMPCjrSound {
    #[wasm_bindgen(js_name = "toneVoices", getter)]
    pub fn js_tone_voices(&self) -> Vec<IBMPCjrToneVoice> {
        self.tone_voices.to_vec()
    }
}
