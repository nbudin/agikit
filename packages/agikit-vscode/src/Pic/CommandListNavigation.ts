import React, { useCallback, useMemo } from 'react';
import { EditingPictureCommand, EditingPictureResource } from './EditingPictureTypes';

export type CommandListNavigationContextValue = {
  enabledCommands: EditingPictureCommand[];
  currentCommandId: string | undefined;
  setAllCommandsEnabled: (enabled: boolean) => void;
  setCommandEnabled: (uuid: string, enabled: boolean) => void;
  jumpTo: (uuid: string) => void;
  jumpRelative: (amount: number) => void;
};

export const CommandListNavigationContext = React.createContext<CommandListNavigationContextValue>({
  enabledCommands: [],
  currentCommandId: undefined,
  setAllCommandsEnabled: () => {},
  setCommandEnabled: () => {},
  jumpTo: () => {},
  jumpRelative: () => {},
});

export function useCommandListNavigation(
  commands: EditingPictureResource['commands'],
  setCommands: React.Dispatch<
    (prevCommands: EditingPictureResource['commands']) => EditingPictureResource['commands']
  >,
): CommandListNavigationContextValue {
  const enabledCommands = useMemo(() => commands.filter((c) => c.enabled), [commands]);

  const currentCommandId = useMemo(() => {
    if (enabledCommands.length > 0) {
      return enabledCommands[enabledCommands.length - 1].uuid;
    }
    return undefined;
  }, [enabledCommands]);

  const setAllCommandsEnabled = useCallback(
    (enabled: boolean) => {
      setCommands((prevCommands) =>
        prevCommands.map((command) => ({
          ...command,
          enabled,
        })),
      );
    },
    [setCommands],
  );

  const setCommandEnabled = useCallback(
    (uuid: string, enabled: boolean) => {
      setCommands((prevCommands) =>
        prevCommands.map((command) => {
          if (command.uuid === uuid) {
            return { ...command, enabled };
          }

          return command;
        }),
      );
    },
    [setCommands],
  );

  const jumpTo = useCallback(
    (uuid: string) => {
      setCommands((prevCommands) => {
        const disableAfterIndex = prevCommands.findIndex((command) => command.uuid === uuid);
        return prevCommands.map((command, index) => ({
          ...command,
          enabled: index > disableAfterIndex ? false : true,
        }));
      });
    },
    [setCommands],
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

  const contextValue = useMemo(() => {
    return {
      enabledCommands,
      currentCommandId,
      setAllCommandsEnabled,
      setCommandEnabled,
      jumpTo,
      jumpRelative,
    };
  }, [
    enabledCommands,
    currentCommandId,
    setAllCommandsEnabled,
    setCommandEnabled,
    jumpTo,
    jumpRelative,
  ]);

  return contextValue;
}
