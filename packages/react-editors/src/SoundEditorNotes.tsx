import React from 'react';
import { IBMPCjrNoiseVoice, IBMPCjrToneVoice } from '../../core/dist/Types/Sound';
import { getMIDINoteNumber, getNoteName } from './SoundEditorUtils';

export const ToneVoiceNotes = React.memo(
  ({
    voiceIndex,
    toneVoice,
    timeZoom,
  }: {
    voiceIndex: number;
    toneVoice: IBMPCjrToneVoice;
    timeZoom: number;
  }) => {
    return (
      <>
        {toneVoice.notes.map((note, noteIndex) => {
          const midiNoteNumber = getMIDINoteNumber(note.frequency);
          return (
            <div
              key={noteIndex}
              className={`sound-editor-timeline-note bg-tone-voice-${voiceIndex + 1}`}
              style={{
                width: `${note.duration * timeZoom}px`,
                left: `${note.startTime * timeZoom}px`,
                top: `${(108 - midiNoteNumber) * 20}px`,
              }}
              title={`${note.frequency}hz (${getNoteName(midiNoteNumber)}) for ${
                note.duration / 60
              }s`}
              ref={
                noteIndex === 0 && voiceIndex === 0
                  ? (element) => {
                      element?.scrollIntoView();
                    }
                  : undefined
              }
            ></div>
          );
        })}
      </>
    );
  },
);

export const NoiseVoiceNotes = React.memo(
  ({ noiseVoice, timeZoom }: { noiseVoice: IBMPCjrNoiseVoice; timeZoom: number }) => {
    return (
      <>
        {noiseVoice.notes.map((note, noteIndex) => {
          const midiNoteNumber = getMIDINoteNumber(note.frequency);
          return (
            <div
              key={noteIndex}
              className={`sound-editor-timeline-note bg-noise-voice-${note.noiseType}`}
              style={{
                width: `${note.duration * timeZoom}px`,
                left: `${note.startTime * timeZoom}px`,
                top: `${(108 - midiNoteNumber) * 20}px`,
              }}
              title={`${note.frequency}hz ${note.noiseType} noise for ${note.duration / 60}s`}
            ></div>
          );
        })}
      </>
    );
  },
);
