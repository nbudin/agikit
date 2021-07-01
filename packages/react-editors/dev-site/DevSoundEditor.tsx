import { readIBMPCjrSoundResource } from '@agikit/core/dist/Extract/Sound/ReadSound';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  IBMPCjrNoteCommon,
  IBMPCjrSound,
  IBMPCjrToneNote,
  IBMPCjrToneVoice,
} from '@agikit/core/dist/Types/Sound';
import { isPresent } from 'ts-is-present';
import { operationReconThemeBase64 } from './dev-example-data';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
import '../styles/soundeditor.css';

const operationReconTheme = readIBMPCjrSoundResource(
  Buffer.from(operationReconThemeBase64, 'base64'),
);

function gcd(a: number, b: number) {
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

function getRealNoteDurations(voice: { notes: IBMPCjrNoteCommon[] }): number[] {
  return voice.notes
    .map((note) => note.duration)
    .filter((duration) => !Number.isNaN(duration) && Number.isFinite(duration));
}

function findBeatDuration(sound: IBMPCjrSound) {
  const shortestNoteDurations = [
    ...sound.toneVoices.map((voice) => Math.min(...getRealNoteDurations(voice))),
    Math.min(...getRealNoteDurations(sound.noiseVoice)),
  ].filter((duration) => !Number.isNaN(duration) && Number.isFinite(duration));

  return shortestNoteDurations.reduce(gcd);
}

function getSemitonesFromA4(frequency: number): number {
  return 12 * Math.log2(frequency / 440);
}

function getMIDINoteNumber(frequency: number): number {
  return Math.round(getSemitonesFromA4(frequency)) + 69;
}

function getNoteName(noteNumber: number): string {
  const octave = Math.floor((noteNumber - 12) / 12);
  const noteInOctave = noteNumber % 12;
  const noteName = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'][noteInOctave];
  return `${noteName}${octave}`;
}

const NOTE_DURATION_UNITS_PER_SECOND = 60;
const SCHEDULE_AHEAD_SECONDS = 0.1;
const LOOKAHEAD_MS = 25.0;

class TickEvent extends Event {
  currentTime: number;
  startTime: number;
  playheadOffset: number;

  constructor(currentTime: number, startTime: number, playheadOffset: number) {
    super('tick');
    this.currentTime = currentTime;
    this.startTime = startTime;
    this.playheadOffset = playheadOffset;
  }

  get currentTimeAbsolute() {
    return this.startTime + this.currentTime;
  }
}

class SeekEvent extends Event {
  newTime: number;

  constructor(newTime: number) {
    super('seek');
    this.newTime = newTime;
  }
}

class IBMPCjrToneVoicePlayer {
  private ctx: AudioContext;
  private voice: IBMPCjrToneVoice;
  private currentNoteIndex: number;
  private nextNoteTime: number | undefined;
  volume: number;

  constructor(soundPlayer: IBMPCjrSoundPlayer, ctx: AudioContext, voice: IBMPCjrToneVoice) {
    this.ctx = ctx;
    this.voice = voice;
    this.currentNoteIndex = 0;
    this.volume = 0.05;

    soundPlayer.addEventListener('play', () => {
      this.nextNoteTime = this.voice.notes[this.currentNoteIndex]?.startTime;
    });

    soundPlayer.addEventListener('tick', (event: Event) => {
      this.onTick(event as TickEvent);
    });

    soundPlayer.addEventListener('seek', (event: Event) => {
      this.onSeek(event as SeekEvent);
    });
  }

  onTick(event: TickEvent) {
    while (
      this.nextNoteTime != null &&
      this.nextNoteTime < event.currentTime * 60 + SCHEDULE_AHEAD_SECONDS
    ) {
      const currentNote = this.voice.notes[this.currentNoteIndex];
      if (!currentNote) {
        break;
      }

      this.scheduleNote(
        currentNote,
        event.startTime + this.nextNoteTime / 60 - event.playheadOffset,
      );
      this.currentNoteIndex += 1;
      this.nextNoteTime = this.voice.notes[this.currentNoteIndex]?.startTime;
    }
  }

  onSeek(event: SeekEvent) {
    const newTimeInPCjrUnits = event.newTime * 60;
    const firstNoteIndex = this.voice.notes.findIndex(
      (note) => note.startTime >= newTimeInPCjrUnits,
    );
    if (firstNoteIndex < 0) {
      this.currentNoteIndex = 0;
    } else {
      this.currentNoteIndex = firstNoteIndex;
    }
  }

  private scheduleNote(note: IBMPCjrToneNote, startTime: number) {
    if (note.attenuation >= 15) {
      // 15 = don't make a sound
      return;
    }

    const oscillator = this.ctx.createOscillator();
    oscillator.type = 'square';
    oscillator.frequency.value = note.frequency;

    // -1 * attenuation = 20 * log10(gain)
    // (-1 * attenuation) / 20 = log10(gain)
    // 10 ** ((-1 * attenuation) / 20) = gain
    const gain = this.ctx.createGain();
    gain.gain.value = 10 ** ((-1 * note.attenuation) / 20) * this.volume;

    oscillator.connect(gain);
    gain.connect(this.ctx.destination);

    oscillator.start(startTime);
    oscillator.stop(startTime + note.duration / NOTE_DURATION_UNITS_PER_SECOND);
  }
}

class IBMPCjrSoundPlayer extends EventTarget {
  private ctx: AudioContext;
  private sound: IBMPCjrSound;
  private tonePlayers: [IBMPCjrToneVoicePlayer, IBMPCjrToneVoicePlayer, IBMPCjrToneVoicePlayer];
  private _volume: number;
  private timerId: number | undefined;
  private startTime: number | undefined;
  private playheadOffset: number;
  playing: boolean;

  constructor(ctx: AudioContext, sound: IBMPCjrSound) {
    super();
    this.ctx = ctx;
    this.sound = sound;
    this._volume = 0.05;
    this.playing = false;
    this.playheadOffset = 0;

    this.tonePlayers = [
      new IBMPCjrToneVoicePlayer(this, ctx, sound.toneVoices[0]),
      new IBMPCjrToneVoicePlayer(this, ctx, sound.toneVoices[1]),
      new IBMPCjrToneVoicePlayer(this, ctx, sound.toneVoices[2]),
    ];

    this.volume = this._volume; // set initial volume on players
  }

  get volume(): number {
    return this._volume;
  }

  set volume(newVolume: number) {
    this._volume = newVolume;
    this.tonePlayers.forEach((player) => (player.volume = newVolume));
  }

  play() {
    this.playing = true;
    this.dispatchEvent(new Event('play'));
    this.ctx.resume();
    this.startTime = this.ctx.currentTime;
    this.tick();
  }

  pause() {
    this.playing = false;
    this.dispatchEvent(new Event('pause'));
    this.playheadOffset = this.ctx.currentTime - (this.startTime ?? 0);
    this.ctx.suspend();

    if (this.timerId) {
      window.clearTimeout(this.timerId);
      this.timerId = undefined;
    }
  }

  seek(newTime: number) {
    this.playheadOffset = newTime;
    this.startTime = this.ctx.currentTime;
    this.dispatchEvent(new SeekEvent(newTime));
  }

  private tick() {
    this.dispatchEvent(
      new TickEvent(
        this.ctx.currentTime - (this.startTime ?? 0) + this.playheadOffset,
        this.startTime ?? 0,
        this.playheadOffset,
      ),
    );
    this.timerId = window.setTimeout(() => this.tick(), LOOKAHEAD_MS);
  }
}

function DevSoundEditor() {
  const sound = operationReconTheme;
  const [playing, setPlaying] = useState(false);
  const [currentTime, setCurrentTime] = useState(0);
  const [timeZoom, setTimeZoom] = useState(3);
  const timelineContainerDivRef = useRef<HTMLDivElement>(null);
  const timelineDivRef = useRef<HTMLDivElement>(null);

  const player = useMemo(() => {
    const ctx = new AudioContext();
    return new IBMPCjrSoundPlayer(ctx, sound);
  }, [sound]);

  const onPlay = useCallback(() => {
    setPlaying(true);
  }, []);
  const onPause = useCallback(() => {
    setPlaying(false);
  }, []);
  const onTick = useCallback((event: Event) => {
    setCurrentTime((event as TickEvent).currentTime);
  }, []);
  const onSeek = useCallback((event: Event) => {
    setCurrentTime((event as SeekEvent).newTime);
  }, []);

  useEffect(() => {
    player.addEventListener('play', onPlay);
    player.addEventListener('pause', onPause);
    player.addEventListener('tick', onTick);
    player.addEventListener('seek', onSeek);
    return () => {
      player.removeEventListener('play', onPlay);
      player.removeEventListener('pause', onPause);
      player.removeEventListener('tick', onTick);
      player.removeEventListener('seek', onSeek);
    };
  }, [player, onTick, onPause, onPlay, onSeek]);

  const timestamp = useMemo(() => {
    const roundedTime = Math.floor(currentTime);
    return `${Math.floor(roundedTime / 60)}:${(roundedTime % 60).toString().padStart(2, '0')}`;
  }, [currentTime]);

  const soundLength = useMemo(() => {
    const lastNotes = [...sound.toneVoices, sound.noiseVoice]
      .map((voice) => voice.notes[voice.notes.length - 1])
      .filter(isPresent);
    return Math.max(...lastNotes.map((note) => note.duration + note.startTime));
  }, [sound]);

  const clickInTimeline = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      const timelineDiv = timelineDivRef.current;
      if (!timelineDiv) {
        return;
      }

      const timelineX =
        event.clientX - timelineDiv.offsetLeft + (timelineDiv.parentElement?.scrollLeft ?? 0);
      const seekTimePCjrUnits = timelineX / timeZoom;
      const seekTime = seekTimePCjrUnits / 60;
      player.seek(seekTime);
    },
    [timeZoom, player],
  );

  return (
    <div className="sound-editor">
      <div className="sound-editor-toolbar">
        <button
          onClick={() => {
            if (playing) {
              player.pause();
            } else {
              player.play();
            }
          }}
          aria-label={playing ? 'Pause' : 'Play'}
        >
          <i className={`agikit-tool-button primary ${playing ? `bi-pause` : 'bi-play'}`} />
        </button>{' '}
        {timestamp}
      </div>
      <div className="sound-editor-timeline-container" ref={timelineContainerDivRef}>
        <div
          className="sound-editor-timeline"
          style={{
            width: `${soundLength * timeZoom}px`,
            cursor: 'text',
          }}
          ref={timelineDivRef}
          onClick={clickInTimeline}
        >
          {sound.toneVoices.map((toneVoice, voiceIndex) => (
            <ToneVoiceNotes
              voiceIndex={voiceIndex}
              toneVoice={toneVoice}
              timeZoom={timeZoom}
              key={voiceIndex}
            />
          ))}
          <div
            className="sound-editor-timeline-playhead"
            style={{
              left: `${currentTime * 60 * timeZoom}px`,
            }}
          ></div>
        </div>
      </div>
    </div>
  );
}

window.addEventListener('load', () => {
  ReactDOM.render(<DevSoundEditor />, document.getElementById('sound-editor-root'));
});

const ToneVoiceNotes = React.memo(
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
