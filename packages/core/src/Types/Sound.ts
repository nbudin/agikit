export type IBMPCjrNoteCommon = {
  startTime: number;
  duration: number;
  frequency: number;
  attenuation: number;
};

export type IBMPCjrToneNote = IBMPCjrNoteCommon;

export type IBMPCjrNoiseNote = IBMPCjrNoteCommon & {
  noiseType: 'white' | 'periodic';
};

export type IBMPCjrToneVoice = {
  notes: IBMPCjrToneNote[];
};

export type IBMPCjrNoiseVoice = {
  notes: IBMPCjrNoiseNote[];
};

export type IBMPCjrSound = {
  toneVoices: [IBMPCjrToneVoice, IBMPCjrToneVoice, IBMPCjrToneVoice];
  noiseVoice: IBMPCjrNoiseVoice;
};
