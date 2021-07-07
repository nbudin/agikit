import { encodeUInt16LE } from '../DataEncoding';
import {
  IBMPCjrNoiseNote,
  IBMPCjrNoiseVoice,
  IBMPCjrNoteCommon,
  IBMPCjrSound,
  IBMPCjrToneNote,
  IBMPCjrToneVoice,
} from '../Types/Sound';

function insertSilentNotes<NoteType extends IBMPCjrNoteCommon>(
  notes: NoteType[],
  buildSilentNote: (common: IBMPCjrNoteCommon) => NoteType,
): NoteType[] {
  const notesWithSilences: NoteType[] = [];
  let currentTime = 0;
  notes.forEach((note) => {
    if (note.startTime > currentTime) {
      notesWithSilences.push(
        buildSilentNote({
          startTime: currentTime,
          attenuation: 15,
          duration: note.startTime - currentTime,
          frequency: Infinity,
        }),
      );
      currentTime = note.startTime;
    }
    notesWithSilences.push(note);
    currentTime += note.duration;
  });
  return notesWithSilences;
}

function buildIBMPCjrToneNote(
  note: IBMPCjrToneNote,
  frequencyRegister: number,
  attenuationRegister: number,
): Buffer {
  const frequencyDivisor = Math.floor(111860 / note.frequency);
  const frequencyDivisorByte1 = (frequencyDivisor >> 4) & 0b00111111;
  const frequencyDivisorByte2 = 0b10000000 + (frequencyRegister << 4) + (frequencyDivisor & 0x0f);
  const attenuationByte = 0b10000000 + (attenuationRegister << 4) + note.attenuation;

  if (frequencyRegister === 2) {
    console.log(frequencyDivisorByte2);
  }

  return Buffer.from([
    ...encodeUInt16LE(note.duration),
    frequencyDivisorByte1,
    frequencyDivisorByte2,
    attenuationByte,
  ]);
}

function buildIBMPCjrNoiseNote(note: IBMPCjrNoiseNote): Buffer {
  const frequencyDivisor = Math.floor(1193180 / note.frequency);
  const frequencyDivisorLogMinus9 = Math.floor(Math.log2(frequencyDivisor)) - 9;
  const noiseTypeBit = note.noiseType === 'white' ? 0b100 : 0;
  const frequencyDivisorByte = 0b10000000 + (0b110 << 4) + noiseTypeBit + frequencyDivisorLogMinus9;
  const attenuationByte = 0b10000000 + (0b111 << 4) + note.attenuation;

  return Buffer.from([...encodeUInt16LE(note.duration), 0, frequencyDivisorByte, attenuationByte]);
}

function buildIBMPCjrToneVoice(voice: IBMPCjrToneVoice, voiceIndex: number): Buffer {
  const notesWithSilences = insertSilentNotes<IBMPCjrToneNote>(voice.notes, (common) => common);
  const frequencyRegister = voiceIndex * 2;
  const attenuationRegister = frequencyRegister + 1;
  return Buffer.concat([
    ...notesWithSilences.map((note) =>
      buildIBMPCjrToneNote(note, frequencyRegister, attenuationRegister),
    ),
    Buffer.from([0xff, 0xff]),
  ]);
}

function buildIBMPCjrNoiseVoice(voice: IBMPCjrNoiseVoice): Buffer {
  const notesWithSilences = insertSilentNotes<IBMPCjrNoiseNote>(voice.notes, (common) => ({
    ...common,
    noiseType: 'white',
  }));
  return Buffer.concat([
    ...notesWithSilences.map((note) => buildIBMPCjrNoiseNote(note)),
    Buffer.from([0xff, 0xff]),
  ]);
}

export function buildIBMPCjrSound(sound: IBMPCjrSound): Buffer {
  const toneVoices = [
    buildIBMPCjrToneVoice(sound.toneVoices[0], 0),
    buildIBMPCjrToneVoice(sound.toneVoices[1], 1),
    buildIBMPCjrToneVoice(sound.toneVoices[2], 2),
  ];
  const noiseVoice = buildIBMPCjrNoiseVoice(sound.noiseVoice);

  let offset = 8; // header is 8 bytes
  const header: number[] = [];
  [...toneVoices, noiseVoice].forEach((buffer) => {
    header.push(...encodeUInt16LE(offset));
    offset += buffer.byteLength;
  });

  return Buffer.concat([Buffer.from(header), ...toneVoices, noiseVoice]);
}
