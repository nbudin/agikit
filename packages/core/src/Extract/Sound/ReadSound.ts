import {
  IBMPCjrNoiseNote,
  IBMPCjrNoiseVoice,
  IBMPCjrSound,
  IBMPCjrToneNote,
  IBMPCjrToneVoice,
} from '../../Types/Sound';

export function readIBMPCjrSoundResource(data: Buffer): IBMPCjrSound {
  let offset = 0;

  const consumeUInt8 = () => {
    const value = data.readUInt8(offset);
    offset += 1;
    return value;
  };

  const consumeUInt16LE = () => {
    const value = data.readUInt16LE(offset);
    offset += 2;
    return value;
  };

  const readToneVoice = (startingOffset: number): IBMPCjrToneVoice => {
    offset = startingOffset;

    const notes: IBMPCjrToneNote[] = [];
    let startTime = 0;
    while (offset < data.byteLength) {
      const duration = consumeUInt16LE();
      if (duration === 0xffff) {
        // FFFF signifies end of voice
        break;
      }

      const frequencyDivisorByte1 = consumeUInt8();
      const frequencyDivisorByte2 = consumeUInt8();
      const frequency = Math.floor(
        111860 / (((frequencyDivisorByte1 & 0x3f) << 4) + (frequencyDivisorByte2 & 0x0f)),
      );

      const attenuationByte = consumeUInt8();
      const attenuation = attenuationByte & 0x0f;

      // 15 = voice off, we don't want to treat these as notes
      if (attenuation < 15) {
        notes.push({ attenuation, frequency, duration, startTime });
      }
      startTime += duration;
    }

    return { notes };
  };

  const readNoiseVoice = (startingOffset: number): IBMPCjrNoiseVoice => {
    offset = startingOffset;

    const notes: IBMPCjrNoiseNote[] = [];
    let startTime = 0;
    while (offset < data.byteLength) {
      const duration = consumeUInt16LE();
      if (duration === 0xffff) {
        // FFFF signifies end of voice
        break;
      }

      offset += 1; // third byte is always zero for the noise voice
      const frequencyByte = consumeUInt8();
      const noiseType = (frequencyByte & 0b100) === 0b100 ? 'white' : 'periodic';
      const frequencyDivisorLogMinus9 = frequencyByte & 0b11;
      const frequency = Math.floor(1193180 / 2 ** (frequencyDivisorLogMinus9 + 9));

      const attenuationByte = consumeUInt8();
      const attenuation = attenuationByte & 0x0f;

      // 15 = voice off, we don't want to treat these as notes
      if (attenuation < 15) {
        notes.push({
          noiseType,
          attenuation,
          duration,
          frequency,
          startTime,
        });
      }
      startTime += duration;
    }

    return { notes };
  };

  const toneVoiceOffsets = [consumeUInt16LE(), consumeUInt16LE(), consumeUInt16LE()] as const;
  const noiseVoiceOffset = consumeUInt16LE();

  const toneVoices: [IBMPCjrToneVoice, IBMPCjrToneVoice, IBMPCjrToneVoice] = [
    readToneVoice(toneVoiceOffsets[0]),
    readToneVoice(toneVoiceOffsets[1]),
    readToneVoice(toneVoiceOffsets[2]),
  ];
  const noiseVoice = readNoiseVoice(noiseVoiceOffset);

  return {
    toneVoices,
    noiseVoice,
  };
}
