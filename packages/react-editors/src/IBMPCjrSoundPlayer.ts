import { IBMPCjrSound, IBMPCjrToneNote, IBMPCjrToneVoice } from '@agikit/core/dist/Types/Sound';
import { isPresent } from 'ts-is-present';

const NOTE_DURATION_UNITS_PER_SECOND = 60;
const SCHEDULE_AHEAD_SECONDS = 0.1;
const LOOKAHEAD_MS = 1 / NOTE_DURATION_UNITS_PER_SECOND;

export class TickEvent extends Event {
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

export class SeekEvent extends Event {
  newTime: number;

  constructor(newTime: number) {
    super('seek');
    this.newTime = newTime;
  }
}

export class IBMPCjrToneVoicePlayer {
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
    this.nextNoteTime = this.voice.notes[this.currentNoteIndex]?.startTime;
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

export default class IBMPCjrSoundPlayer extends EventTarget {
  private ctx: AudioContext;
  private sound: IBMPCjrSound;
  private tonePlayers: [IBMPCjrToneVoicePlayer, IBMPCjrToneVoicePlayer, IBMPCjrToneVoicePlayer];
  private _volume: number;
  private timerId: number | undefined;
  private startTime: number | undefined;
  private playheadOffset: number;
  readonly soundLength: number;
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

    const lastNotes = [...sound.toneVoices, sound.noiseVoice]
      .map((voice) => voice.notes[voice.notes.length - 1])
      .filter(isPresent);
    this.soundLength = Math.max(...lastNotes.map((note) => note.duration + note.startTime));
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

    if (this.playheadOffset >= this.soundLength / 60) {
      this.seek(0);
    }

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
    const currentTimeRelative = this.ctx.currentTime - (this.startTime ?? 0) + this.playheadOffset;
    this.dispatchEvent(
      new TickEvent(currentTimeRelative, this.startTime ?? 0, this.playheadOffset),
    );
    if (currentTimeRelative >= this.soundLength / 60) {
      this.pause();
    } else {
      this.timerId = window.setTimeout(() => this.tick(), LOOKAHEAD_MS);
    }
  }
}
