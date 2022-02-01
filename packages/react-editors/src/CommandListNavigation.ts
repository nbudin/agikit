import {
  DEFAULT_PEN_SETTINGS,
  ChangePenPictureCommand,
  DisablePictureDrawPictureCommand,
  DisablePriorityDrawPictureCommand,
  PicturePenSettings,
  SetPictureColorPictureCommand,
  SetPriorityColorPictureCommand,
} from '@agikit/core';
import React, { useCallback, useContext, useMemo } from 'react';
import { EditingPictureCommand, EditingPictureResource } from './EditingPictureTypes';
import { PicEditorControlContext } from './PicEditorControlContext';

export type CommandListNavigationContextValue = {
  currentCommandColors: {
    visual: number | undefined;
    priority: number | undefined;
  };
  currentCommandPenSettings: PicturePenSettings;
  enabledCommands: EditingPictureCommand[];
  currentCommandId: string | undefined;
  setAllCommandsEnabled: (enabled: boolean) => void;
  setCommandEnabled: (uuid: string, enabled: boolean) => void;
  jumpTo: (uuid: string) => void;
  jumpRelative: (amount: number) => void;
};

export const CommandListNavigationContext = React.createContext<CommandListNavigationContextValue>({
  currentCommandColors: {
    visual: undefined,
    priority: undefined,
  },
  currentCommandPenSettings: DEFAULT_PEN_SETTINGS,
  enabledCommands: [],
  currentCommandId: undefined,
  setAllCommandsEnabled: () => {},
  setCommandEnabled: () => {},
  jumpTo: () => {},
  jumpRelative: () => {},
});

export function useCommandListNavigation(
  commands: EditingPictureResource['commands'],
): CommandListNavigationContextValue {
  const enabledCommands = useMemo(() => commands.filter((c) => c.enabled), [commands]);
  const { setCommandsEnabled } = useContext(PicEditorControlContext);

  const currentCommandId = useMemo(() => {
    if (enabledCommands.length > 0) {
      return enabledCommands[enabledCommands.length - 1]!.uuid;
    }
    return undefined;
  }, [enabledCommands]);

  const setAllCommandsEnabled = useCallback(
    (enabled: boolean) => setCommandsEnabled(() => enabled),
    [setCommandsEnabled],
  );

  const setCommandEnabled = useCallback(
    (uuid: string, enabled: boolean) => {
      setCommandsEnabled((command) => {
        if (command.uuid === uuid) {
          return enabled;
        }

        return command.enabled;
      });
    },
    [setCommandsEnabled],
  );

  const jumpTo = useCallback(
    (uuid: string) => {
      let foundCommand = false;
      setCommandsEnabled((command) => {
        const enabled = foundCommand ? false : true;
        if (command.uuid === uuid) {
          foundCommand = true;
        }
        return enabled;
      });
    },
    [setCommandsEnabled],
  );

  const jumpRelative = useCallback(
    (amount: number) => {
      const currentCommandIndex =
        currentCommandId != null ? commands.findIndex((c) => c.uuid === currentCommandId) : -1;
      const targetCommand = commands[currentCommandIndex + amount];
      if (targetCommand) {
        jumpTo(targetCommand.uuid);
      }
    },
    [jumpTo, currentCommandId, commands],
  );

  const [currentCommandColors, currentCommandPenSettings] = useMemo(() => {
    const currentCommandIndex =
      currentCommandId != null ? enabledCommands.findIndex((c) => c.uuid === currentCommandId) : 0;
    const currentCommand = enabledCommands[currentCommandIndex];
    if (!currentCommand) {
      return [{ visual: undefined, priority: undefined }, DEFAULT_PEN_SETTINGS];
    }

    const reversedCommandsThroughCurrent = enabledCommands
      .slice(0, currentCommandIndex + 1)
      .reverse();
    let visualCommand: SetPictureColorPictureCommand | DisablePictureDrawPictureCommand | undefined;
    let priorityCommand:
      | SetPriorityColorPictureCommand
      | DisablePriorityDrawPictureCommand
      | undefined;
    let changePenCommand: ChangePenPictureCommand | undefined;
    for (let command of reversedCommandsThroughCurrent) {
      if (
        !visualCommand &&
        (command.type === 'SetPictureColor' || command.type === 'DisablePictureDraw')
      ) {
        visualCommand = command;
      }

      if (
        !priorityCommand &&
        (command.type === 'SetPriorityColor' || command.type === 'DisablePriorityDraw')
      ) {
        priorityCommand = command;
      }

      if (!changePenCommand && command.type === 'ChangePen') {
        changePenCommand = command;
      }

      if (visualCommand && priorityCommand && changePenCommand) {
        break;
      }
    }

    const colors = {
      visual: visualCommand?.type === 'SetPictureColor' ? visualCommand.colorNumber : undefined,
      priority:
        priorityCommand?.type === 'SetPriorityColor' ? priorityCommand.colorNumber : undefined,
    };

    return [colors, changePenCommand?.settings ?? DEFAULT_PEN_SETTINGS];
  }, [currentCommandId, enabledCommands]);

  const contextValue = useMemo(() => {
    return {
      currentCommandColors,
      currentCommandPenSettings,
      enabledCommands,
      currentCommandId,
      setAllCommandsEnabled,
      setCommandEnabled,
      jumpTo,
      jumpRelative,
    };
  }, [
    currentCommandColors,
    currentCommandPenSettings,
    enabledCommands,
    currentCommandId,
    setAllCommandsEnabled,
    setCommandEnabled,
    jumpTo,
    jumpRelative,
  ]);

  return contextValue;
}
