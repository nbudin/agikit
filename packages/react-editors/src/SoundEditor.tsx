import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { IBMPCjrSound } from '@agikit/core';
import IBMPCjrSoundPlayer, { SeekEvent, TickEvent } from './IBMPCjrSoundPlayer';
import { NoiseVoiceNotes, ToneVoiceNotes } from './SoundEditorNotes';

export function SoundEditor({ sound }: { sound: IBMPCjrSound }): JSX.Element {
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

  useEffect(() => {
    const keyDownHandler = (event: KeyboardEvent) => {
      if (event.key === 'Home') {
        player.seek(0);
      } else if (event.key === 'End') {
        player.seek(player.soundLength / 60);
      } else if (event.key === ' ') {
        if (player.playing) {
          player.pause();
        } else {
          player.play();
        }
      } else {
        return;
      }

      // the else return above means we only do this for keys we handled
      event.stopPropagation();
      event.preventDefault();
    };

    window.addEventListener('keydown', keyDownHandler);
    return () => {
      window.removeEventListener('keydown', keyDownHandler);
    };
  }, [player]);

  const timestamp = useMemo(() => {
    const roundedTime = Math.floor(currentTime);
    return `${Math.floor(roundedTime / 60)}:${(roundedTime % 60).toString().padStart(2, '0')}`;
  }, [currentTime]);

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

  const soundLength = player.soundLength;

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
          <NoiseVoiceNotes noiseVoice={sound.noiseVoice} timeZoom={timeZoom} />
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
