import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { renderPicture } from 'agikit-core/dist/Extract/Picture/RenderPicture';
import { assertNever } from 'assert-never';
import { CursorPosition, PicCanvas } from './PicCanvas';
import { EditingPictureCommand, EditingPictureResource } from './EditingPictureTypes';
import { PicCommandList } from './PicCommandList';
import { PictureTool, PICTURE_TOOLS } from './PicEditorTools';
import PicEditorTools from './PicEditorTools';
import {
  ChangePenPictureCommand,
  DisablePictureDrawPictureCommand,
  DisablePriorityDrawPictureCommand,
  PictureCommand,
  PictureCoordinate,
  PicturePenSettings,
  SetPictureColorPictureCommand,
  SetPriorityColorPictureCommand,
} from 'agikit-core/dist/Types/Picture';
import { CommandListNavigationContext, useCommandListNavigation } from './CommandListNavigation';
import { clamp, kebabCase, throttle } from 'lodash';
import { describeCommand } from './describeCommand';

type CommandInProgress = Exclude<
  PictureCommand,
  | SetPictureColorPictureCommand
  | SetPriorityColorPictureCommand
  | DisablePictureDrawPictureCommand
  | DisablePriorityDrawPictureCommand
  | ChangePenPictureCommand
>;

function getInitialCommandForSelectedTool(
  selectedTool: PictureTool,
  penSettings: PicturePenSettings,
  position: CursorPosition,
): CommandInProgress {
  if (selectedTool.name === 'absoluteLine') {
    return {
      type: 'AbsoluteLine',
      opcode: 246,
      points: [position],
    };
  }

  if (selectedTool.name === 'relativeLine') {
    return {
      type: 'RelativeLine',
      opcode: 247,
      startPosition: position,
      relativePoints: [],
    };
  }

  if (selectedTool.name === 'corner') {
    return {
      type: 'DrawXCorner', // might change this to DrawYCorner on second click
      opcode: 245,
      startPosition: position,
      steps: [],
    };
  }

  if (selectedTool.name === 'fill') {
    return {
      type: 'Fill',
      opcode: 248,
      startPositions: [position],
    };
  }

  if (selectedTool.name === 'pen') {
    return {
      type: 'PlotWithPen',
      opcode: 250,
      points: [
        { position, texture: penSettings.splatter ? Math.floor(Math.random() * 120) : undefined },
      ],
    };
  }

  assertNever(selectedTool);
}

function addToCommandInProgress(
  commandInProgress: CommandInProgress,
  penSettings: PicturePenSettings,
  position: CursorPosition,
): CommandInProgress {
  if (commandInProgress.type === 'AbsoluteLine') {
    return {
      ...commandInProgress,
      points: [...commandInProgress.points, position],
    };
  }

  if (commandInProgress.type === 'RelativeLine') {
    const lastPosition = commandInProgress.relativePoints.reduce(
      (pos, relativePoint) => ({
        x: pos.x + relativePoint.x,
        y: pos.y + relativePoint.y,
      }),
      commandInProgress.startPosition,
    );

    const rawX = position.x - lastPosition.x;
    const rawY = position.y - lastPosition.y;
    const relativePoint: PictureCoordinate = {
      x: clamp(rawX, -6, 7),
      y: clamp(rawY, -7, 7),
    };
    return {
      ...commandInProgress,
      relativePoints: [...commandInProgress.relativePoints, relativePoint],
    };
  }

  if (commandInProgress.type === 'DrawXCorner' || commandInProgress.type === 'DrawYCorner') {
    const lastPosition = commandInProgress.steps.reduce(
      (pos, step) => ({ ...pos, [step.axis]: step.position }),
      commandInProgress.startPosition,
    );

    if (commandInProgress.steps.length === 0) {
      // first step could be X or Y
      // we need to figure out what direction we're moving in
      const diffX = position.x - lastPosition.x;
      const diffY = position.y - lastPosition.y;

      if (Math.abs(diffX) > Math.abs(diffY)) {
        return {
          ...commandInProgress,
          type: 'DrawXCorner',
          opcode: 245,
          steps: [{ axis: 'x', position: position.x }],
        };
      } else {
        return {
          ...commandInProgress,
          type: 'DrawYCorner',
          opcode: 244,
          steps: [{ axis: 'y', position: position.y }],
        };
      }
    }

    const axis =
      commandInProgress.steps[commandInProgress.steps.length - 1].axis === 'x' ? 'y' : 'x';
    return {
      ...commandInProgress,
      steps: [...commandInProgress.steps, { axis, position: position[axis] }],
    };
  }

  if (commandInProgress.type === 'Fill') {
    return {
      ...commandInProgress,
      startPositions: [...commandInProgress.startPositions, position],
    };
  }

  if (commandInProgress.type === 'PlotWithPen') {
    return {
      ...commandInProgress,
      points: [
        ...commandInProgress.points,
        { position, texture: penSettings.splatter ? Math.floor(Math.random() * 120) : undefined },
      ],
    };
  }

  assertNever(commandInProgress);
}

export function PicEditor({
  pictureResource,
  setPictureResource,
}: {
  pictureResource: EditingPictureResource;
  setPictureResource: React.Dispatch<React.SetStateAction<EditingPictureResource>>;
}) {
  const [selectedTool, setSelectedTool] = useState<PictureTool>(PICTURE_TOOLS[0]);
  const [visualColor, setVisualColor] = useState<number | undefined>();
  const [priorityColor, setPriorityColor] = useState<number | undefined>();
  const [penSettings, setPenSettings] = useState<PicturePenSettings>({
    shape: 'rectangle',
    size: 0,
    splatter: false,
  });
  const [visualCursorPosition, setVisualCursorPosition] = useState<CursorPosition | undefined>();
  const [priorityCursorPosition, setPriorityCursorPosition] = useState<
    CursorPosition | undefined
  >();
  const [commandInProgress, setCommandInProgress] = useState<CommandInProgress | undefined>();
  const renderedPicture = useMemo(
    () =>
      renderPicture({
        ...pictureResource,
        commands: pictureResource.commands.filter((command) => command.enabled),
      }),
    [pictureResource],
  );
  const commandInProgressWithPreview = useMemo(() => {
    if (commandInProgress) {
      const cursorPosition = visualCursorPosition ?? priorityCursorPosition;
      if (
        cursorPosition &&
        (commandInProgress.type === 'AbsoluteLine' ||
          commandInProgress.type === 'RelativeLine' ||
          commandInProgress.type === 'DrawXCorner' ||
          commandInProgress.type === 'DrawYCorner')
      ) {
        return addToCommandInProgress(commandInProgress, penSettings, cursorPosition);
      }
      return commandInProgress;
    }
  }, [commandInProgress, visualCursorPosition, priorityCursorPosition, penSettings]);
  const renderedPictureWithCommandInProgress = useMemo(() => {
    if (commandInProgressWithPreview) {
      return renderPicture(
        {
          ...pictureResource,
          commands: [commandInProgressWithPreview],
        },
        { renderedPicture, pictureColor: visualColor, priorityColor, pen: penSettings },
      );
    } else {
      return renderedPicture;
    }
  }, [
    renderedPicture,
    pictureResource,
    commandInProgressWithPreview,
    visualColor,
    priorityColor,
    penSettings,
  ]);

  const setVisualCursorPositionThrottled = useMemo(
    () => throttle(setVisualCursorPosition, 100),
    [],
  );

  const setPriorityCursorPositionThrottled = useMemo(
    () => throttle(setPriorityCursorPosition, 100),
    [],
  );

  const setCommands = useCallback(
    (calculateNewState: (prevCommands: EditingPictureCommand[]) => EditingPictureCommand[]) =>
      setPictureResource((prevResource) => ({
        ...prevResource,
        commands: calculateNewState(prevResource.commands),
      })),
    [setPictureResource],
  );

  const navigationContextValue = useCommandListNavigation(pictureResource.commands, setCommands);
  const { enabledCommands, currentCommandId, jumpRelative } = navigationContextValue;

  const cursorDownInCanvas = (position: CursorPosition) => {
    if (commandInProgress) {
      setCommandInProgress(addToCommandInProgress(commandInProgress, penSettings, position));
    } else {
      setCommandInProgress(getInitialCommandForSelectedTool(selectedTool, penSettings, position));
    }
  };

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        event.stopPropagation();
        jumpRelative(1);
      } else if (event.key === 'ArrowUp') {
        event.preventDefault();
        event.stopPropagation();
        jumpRelative(-1);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [jumpRelative]);

  const currentCommandColors = useMemo(() => {
    const currentCommandIndex =
      currentCommandId != null ? enabledCommands.findIndex((c) => c.uuid === currentCommandId) : 0;
    const currentCommand = enabledCommands[currentCommandIndex];
    if (!currentCommand) {
      return { visual: undefined, priority: undefined };
    }

    const reversedCommandsThroughCurrent = enabledCommands
      .slice(0, currentCommandIndex + 1)
      .reverse();
    let visualCommand: SetPictureColorPictureCommand | DisablePictureDrawPictureCommand | undefined;
    let priorityCommand:
      | SetPriorityColorPictureCommand
      | DisablePriorityDrawPictureCommand
      | undefined;
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

      if (visualCommand && priorityCommand) {
        break;
      }
    }

    return {
      visual: visualCommand?.type === 'SetPictureColor' ? visualCommand.colorNumber : undefined,
      priority:
        priorityCommand?.type === 'SetPriorityColor' ? priorityCommand.colorNumber : undefined,
    };
  }, [currentCommandId, enabledCommands]);

  useEffect(() => {
    setVisualColor(currentCommandColors.visual);
  }, [currentCommandColors.visual]);

  useEffect(() => {
    setPriorityColor(currentCommandColors.priority);
  }, [currentCommandColors.priority]);

  return (
    <CommandListNavigationContext.Provider value={navigationContextValue}>
      <div className="pic-editor">
        <div className="pic-editor-visual-area">
          <h3>Visual</h3>
          <PicCanvas
            buffer={renderedPictureWithCommandInProgress.visualBuffer}
            onCursorMove={setVisualCursorPositionThrottled}
            onCursorDown={cursorDownInCanvas}
            onCursorOut={() => {
              setVisualCursorPositionThrottled(undefined);
            }}
          />
          <div className="pic-editor-canvas-status-line">
            {visualCursorPosition ? (
              commandInProgress ? (
                describeCommand(commandInProgress)
              ) : (
                `Cursor position: ${visualCursorPosition.x}, ${visualCursorPosition.y}`
              )
            ) : (
              <>&nbsp;</>
            )}
          </div>
        </div>

        <div className="pic-editor-priority-area">
          <h3>Priority</h3>
          <PicCanvas
            buffer={renderedPictureWithCommandInProgress.priorityBuffer}
            onCursorMove={setPriorityCursorPositionThrottled}
            onCursorDown={cursorDownInCanvas}
            onCursorOut={() => {
              setPriorityCursorPositionThrottled(undefined);
            }}
          />
          <div className="pic-editor-canvas-status-line">
            {priorityCursorPosition ? (
              commandInProgress ? (
                describeCommand(commandInProgress)
              ) : (
                `Cursor position: ${priorityCursorPosition.x}, ${priorityCursorPosition.y}`
              )
            ) : (
              <>&nbsp;</>
            )}
          </div>
        </div>

        <div className="pic-editor-controls" style={{ display: 'flex', flexDirection: 'column' }}>
          <h3>Tools</h3>
          <PicEditorTools
            selectedTool={selectedTool}
            setSelectedTool={setSelectedTool}
            visualColor={visualColor}
            setVisualColor={setVisualColor}
            priorityColor={priorityColor}
            setPriorityColor={setPriorityColor}
          />
          <hr />
          <h3>Command list</h3>
          <PicCommandList pictureResource={pictureResource} />
        </div>
      </div>
    </CommandListNavigationContext.Provider>
  );
}
