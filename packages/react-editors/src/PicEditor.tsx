import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { renderPicture } from '@agikit/core/dist/Extract/Picture/RenderPicture';
import { assertNever } from 'assert-never';
import { PicCanvas } from './PicCanvas';
import { EditingPictureResource, preparePicCommandForEditing } from './EditingPictureTypes';
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
} from '@agikit/core/dist/Types/Picture';
import { CommandListNavigationContext, useCommandListNavigation } from './CommandListNavigation';
import { clamp, throttle } from 'lodash';
import { describeCommand } from './describeCommand';
import { EGAPalette } from '@agikit/core/dist/ColorPalettes';
import { PicEditorControlContext } from './PicEditorControlContext';
import { CursorPosition } from './DrawingCanvas';

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
      points: [{ position, texture: penSettings.splatter ? generateRandomTexture() : undefined }],
    };
  }

  assertNever(selectedTool);
}

function generateRandomTexture(): number {
  return Math.floor(Math.random() * 120);
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
      commandInProgress.steps[commandInProgress.steps.length - 1]!.axis === 'x' ? 'y' : 'x';
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

export function PicEditor({ pictureResource }: { pictureResource: EditingPictureResource }) {
  const { addCommands } = useContext(PicEditorControlContext);
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
      renderPicture(
        {
          ...pictureResource,
          commands: pictureResource.commands.filter((command) => command.enabled),
        },
        EGAPalette,
      ),
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
          commandInProgress.type === 'DrawYCorner' ||
          commandInProgress.type === 'PlotWithPen')
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
        EGAPalette,
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

  const setVisualCursorPositionThrottled = useMemo(() => throttle(setVisualCursorPosition, 16), []);

  const setPriorityCursorPositionThrottled = useMemo(
    () => throttle(setPriorityCursorPosition, 16),
    [],
  );

  const navigationContextValue = useCommandListNavigation(pictureResource.commands);
  const {
    currentCommandColors,
    currentCommandPenSettings,
    currentCommandId,
    jumpRelative,
  } = navigationContextValue;

  const cursorDownInCanvas = (position: CursorPosition) => {
    if (commandInProgress) {
      setCommandInProgress(addToCommandInProgress(commandInProgress, penSettings, position));
    } else {
      setCommandInProgress(getInitialCommandForSelectedTool(selectedTool, penSettings, position));
    }
  };

  const commitCommandInProgress = useCallback(() => {
    if (!commandInProgress) {
      return;
    }

    const commandsToInsert: PictureCommand[] = [commandInProgress];
    if (currentCommandColors.visual !== visualColor) {
      if (visualColor == null) {
        commandsToInsert.unshift({
          type: 'DisablePictureDraw',
          opcode: 241,
        });
      } else {
        commandsToInsert.unshift({
          type: 'SetPictureColor',
          opcode: 240,
          colorNumber: visualColor,
        });
      }
    }

    if (currentCommandColors.priority !== priorityColor) {
      if (priorityColor == null) {
        commandsToInsert.unshift({
          type: 'DisablePriorityDraw',
          opcode: 243,
        });
      } else {
        commandsToInsert.unshift({
          type: 'SetPriorityColor',
          opcode: 242,
          colorNumber: priorityColor,
        });
      }
    }

    if (
      currentCommandPenSettings.shape !== penSettings.shape ||
      currentCommandPenSettings.size !== penSettings.size ||
      currentCommandPenSettings.splatter !== penSettings.splatter
    ) {
      commandsToInsert.unshift({
        type: 'ChangePen',
        opcode: 249,
        settings: penSettings,
      });
    }

    addCommands(commandsToInsert.map(preparePicCommandForEditing), currentCommandId);
    setCommandInProgress(undefined);
  }, [
    commandInProgress,
    currentCommandColors,
    currentCommandPenSettings,
    currentCommandId,
    visualColor,
    priorityColor,
    penSettings,
    addCommands,
  ]);

  const cancelCommandInProgress = useCallback(() => setCommandInProgress(undefined), []);

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
      } else if (event.key === 'Enter') {
        event.preventDefault();
        event.stopPropagation();
        commitCommandInProgress();
      } else if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        cancelCommandInProgress();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [jumpRelative, commitCommandInProgress, cancelCommandInProgress]);

  useEffect(() => {
    setVisualColor(currentCommandColors.visual);
  }, [currentCommandColors.visual]);

  useEffect(() => {
    setPriorityColor(currentCommandColors.priority);
  }, [currentCommandColors.priority]);

  useEffect(() => {
    setPenSettings(currentCommandPenSettings);
  }, [currentCommandPenSettings]);

  useEffect(() => {
    setCommandInProgress((prevCommandInProgress) => {
      if (prevCommandInProgress && prevCommandInProgress.type === 'PlotWithPen') {
        return {
          ...prevCommandInProgress,
          points: prevCommandInProgress.points.map((point) => ({
            ...point,
            texture: penSettings.splatter ? point.texture ?? generateRandomTexture() : undefined,
          })),
        };
      }

      return prevCommandInProgress;
    });
  }, [penSettings]);

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
            commandInProgress={commandInProgress}
            commitCommandInProgress={commitCommandInProgress}
            cancelCommandInProgress={cancelCommandInProgress}
            selectedTool={selectedTool}
            setSelectedTool={setSelectedTool}
            visualColor={visualColor}
            setVisualColor={setVisualColor}
            priorityColor={priorityColor}
            setPriorityColor={setPriorityColor}
            penSettings={penSettings}
            setPenSettings={setPenSettings}
          />
          <hr />
          <h3>Command list</h3>
          <PicCommandList pictureResource={pictureResource} />
        </div>
      </div>
    </CommandListNavigationContext.Provider>
  );
}
