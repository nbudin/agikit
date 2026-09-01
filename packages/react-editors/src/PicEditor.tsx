import { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { assertNever } from 'assert-never';
import { PicCanvas } from './PicCanvas';
import { EditingPictureResource, preparePicCommandForEditing } from './EditingPictureTypes';
import { PicCommandList } from './PicCommandList';
import { PictureTool, PICTURE_TOOLS } from './PicEditorTools';
import PicEditorTools from './PicEditorTools';
import {
  AbsoluteLinePictureCommand,
  ChangePenPictureCommand,
  DisablePictureDrawPictureCommand,
  DisablePriorityDrawPictureCommand,
  DrawXCornerPictureCommand,
  DrawYCornerPictureCommand,
  EGAPalette,
  FillPictureCommand,
  Picture,
  PictureCommand,
  PictureCoordinate,
  PictureCornerStep,
  PictureCornerStepAxis,
  PicturePenPlotPoint,
  PicturePenSettings,
  PlotWithPenPictureCommand,
  RelativeLinePictureCommand,
  RelativeLinePoint,
  renderPicture,
  RenderPictureStartingFromOptions,
  SetPictureColorPictureCommand,
  SetPriorityColorPictureCommand,
} from '@agikit/core';
import { CommandListNavigationContext, useCommandListNavigation } from './CommandListNavigation';
import { clamp, throttle } from 'lodash';
import { describeCommand } from './describeCommand';
import { PicEditorControlContext } from './PicEditorControlContext';
import { CursorPosition } from './DrawingCanvas';

type CommandInProgress =
  | AbsoluteLinePictureCommand
  | RelativeLinePictureCommand
  | DrawXCornerPictureCommand
  | DrawYCornerPictureCommand
  | FillPictureCommand
  | PlotWithPenPictureCommand;

function getInitialCommandForSelectedTool(
  selectedTool: PictureTool,
  penSettings: PicturePenSettings,
  position: CursorPosition,
): CommandInProgress {
  if (selectedTool.name === 'absoluteLine') {
    return new AbsoluteLinePictureCommand([new PictureCoordinate(position.x, position.y)]);
  }

  if (selectedTool.name === 'relativeLine') {
    return new RelativeLinePictureCommand(new PictureCoordinate(position.x, position.y));
  }

  if (selectedTool.name === 'corner') {
    // might change this to DrawYCorner on second click
    return new DrawXCornerPictureCommand(new PictureCoordinate(position.x, position.y));
  }

  if (selectedTool.name === 'fill') {
    return new FillPictureCommand([new PictureCoordinate(position.x, position.y)]);
  }

  if (selectedTool.name === 'pen') {
    return new PlotWithPenPictureCommand([
      new PicturePenPlotPoint(
        new PictureCoordinate(position.x, position.y),
        penSettings.splatter ? generateRandomTexture() : undefined,
      ),
    ]);
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
    return new AbsoluteLinePictureCommand([
      ...commandInProgress.points,
      new PictureCoordinate(position.x, position.y),
    ]);
  }

  if (commandInProgress.type === 'RelativeLine') {
    const lastPosition = commandInProgress.relativePoints.reduce(
      (pos, relativePoint) =>
        new RelativeLinePoint(pos.x + relativePoint.x, pos.y + relativePoint.y),
      commandInProgress.startPosition,
    );

    const rawX = position.x - lastPosition.x;
    const rawY = position.y - lastPosition.y;
    const relativePoint = new PictureCoordinate(clamp(rawX, -6, 7), clamp(rawY, -7, 7));
    return new RelativeLinePictureCommand(commandInProgress.startPosition, [
      ...commandInProgress.relativePoints,
      relativePoint,
    ]);
  }

  if (commandInProgress.type === 'DrawXCorner' || commandInProgress.type === 'DrawYCorner') {
    const lastPosition = commandInProgress.steps.reduce((pos, step) => {
      if (step.axis === PictureCornerStepAxis.X) {
        return new PictureCoordinate(step.position, pos.y);
      } else {
        return new PictureCoordinate(pos.x, step.position);
      }
    }, commandInProgress.startPosition);

    if (commandInProgress.steps.length === 0) {
      // first step could be X or Y
      // we need to figure out what direction we're moving in
      const diffX = position.x - lastPosition.x;
      const diffY = position.y - lastPosition.y;

      if (Math.abs(diffX) > Math.abs(diffY)) {
        return new DrawXCornerPictureCommand(commandInProgress.startPosition, [
          new PictureCornerStep(PictureCornerStepAxis.X, position.x),
        ]);
      } else {
        return new DrawYCornerPictureCommand(commandInProgress.startPosition, [
          new PictureCornerStep(PictureCornerStepAxis.Y, position.y),
        ]);
      }
    }

    const [axis, newPosition] =
      commandInProgress.steps[commandInProgress.steps.length - 1]!.axis === PictureCornerStepAxis.X
        ? [PictureCornerStepAxis.Y, position.y]
        : [PictureCornerStepAxis.X, position.x];

    if (commandInProgress.type === 'DrawXCorner') {
      return new DrawXCornerPictureCommand(commandInProgress.startPosition, [
        ...commandInProgress.steps,
        new PictureCornerStep(axis, newPosition),
      ]);
    } else {
      return new DrawYCornerPictureCommand(commandInProgress.startPosition, [
        ...commandInProgress.steps,
        new PictureCornerStep(axis, newPosition),
      ]);
    }
  }

  if (commandInProgress.type === 'Fill') {
    return new FillPictureCommand([
      ...commandInProgress.startPositions,
      new PictureCoordinate(position.x, position.y),
    ]);
  }

  if (commandInProgress.type === 'PlotWithPen') {
    return new PlotWithPenPictureCommand([
      ...commandInProgress.points,
      new PicturePenPlotPoint(
        new PictureCoordinate(position.x, position.y),
        penSettings.splatter ? Math.floor(Math.random() * 120) : undefined,
      ),
    ]);
  }

  assertNever(commandInProgress);
}

export function PicEditor({ pictureResource }: { pictureResource: EditingPictureResource }) {
  const { addCommands } = useContext(PicEditorControlContext);
  const [selectedTool, setSelectedTool] = useState<PictureTool>(PICTURE_TOOLS[0]);
  const [visualColor, setVisualColor] = useState<number | undefined>();
  const [priorityColor, setPriorityColor] = useState<number | undefined>();
  const [penSettings, setPenSettings] = useState(new PicturePenSettings(0, 'rectangle', false));
  const [visualCursorPosition, setVisualCursorPosition] = useState<CursorPosition | undefined>();
  const [priorityCursorPosition, setPriorityCursorPosition] = useState<
    CursorPosition | undefined
  >();
  const [commandInProgress, setCommandInProgress] = useState<CommandInProgress | undefined>();
  const renderedPicture = useMemo(
    () =>
      renderPicture(
        new Picture(
          pictureResource.commands
            .filter((command) => command.enabled)
            .map((command) => command.command.toPictureCommand()),
        ),
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
        new Picture([commandInProgressWithPreview.toPictureCommand()]),
        EGAPalette,
        new RenderPictureStartingFromOptions(
          renderedPicture,
          visualColor,
          priorityColor,
          penSettings,
        ),
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
  const { currentCommandColors, currentCommandPenSettings, currentCommandId, jumpRelative } =
    navigationContextValue;

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
        commandsToInsert.unshift(new DisablePictureDrawPictureCommand());
      } else {
        commandsToInsert.unshift(new SetPictureColorPictureCommand(visualColor));
      }
    }

    if (currentCommandColors.priority !== priorityColor) {
      if (priorityColor == null) {
        commandsToInsert.unshift(new DisablePriorityDrawPictureCommand());
      } else {
        commandsToInsert.unshift(new SetPriorityColorPictureCommand(priorityColor));
      }
    }

    if (
      currentCommandPenSettings.shape !== penSettings.shape ||
      currentCommandPenSettings.size !== penSettings.size ||
      currentCommandPenSettings.splatter !== penSettings.splatter
    ) {
      commandsToInsert.unshift(new ChangePenPictureCommand(penSettings));
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
        return new PlotWithPenPictureCommand(
          prevCommandInProgress.points.map(
            (point) =>
              new PicturePenPlotPoint(
                point.position,
                penSettings.splatter ? point.texture ?? generateRandomTexture() : undefined,
              ),
          ),
        );
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
