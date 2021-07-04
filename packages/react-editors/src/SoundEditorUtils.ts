import { IBMPCjrNoteCommon, IBMPCjrSound } from '@agikit/core/dist/Types/Sound';

export function gcd(a: number, b: number) {
  let ca = Math.abs(a);
  let cb = Math.abs(b);
  if (cb > ca) {
    const temp = ca;
    ca = cb;
    cb = temp;
  }

  while (true) {
    if (cb === 0) {
      return ca;
    }
    ca %= cb;
    if (ca === 0) {
      return cb;
    }
    cb %= ca;
  }
}

export function getRealNoteDurations(voice: { notes: IBMPCjrNoteCommon[] }): number[] {
  return voice.notes
    .map((note) => note.duration)
    .filter((duration) => !Number.isNaN(duration) && Number.isFinite(duration));
}

export function findBeatDuration(sound: IBMPCjrSound) {
  const shortestNoteDurations = [
    ...sound.toneVoices.map((voice) => Math.min(...getRealNoteDurations(voice))),
    Math.min(...getRealNoteDurations(sound.noiseVoice)),
  ].filter((duration) => !Number.isNaN(duration) && Number.isFinite(duration));

  return shortestNoteDurations.reduce(gcd);
}

export function getSemitonesFromA4(frequency: number): number {
  return 12 * Math.log2(frequency / 440);
}

export function getMIDINoteNumber(frequency: number): number {
  return Math.round(getSemitonesFromA4(frequency)) + 69;
}

export function getNoteName(noteNumber: number): string {
  const octave = Math.floor((noteNumber - 12) / 12);
  const noteInOctave = noteNumber % 12;
  const noteName = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'][noteInOctave];
  return `${noteName}${octave}`;
}
