import React, { useCallback, useEffect, useMemo, useState } from "react";
import { renderPicture } from "agikit-core/dist/Extract/Picture/RenderPicture";
import { CursorPosition, PicCanvas } from "./PicCanvas";
import {
  EditingPictureCommand,
  EditingPictureResource,
} from "./EditingPictureTypes";
import { PicCommandList } from "./PicCommandList";
import { PictureTool, PICTURE_TOOLS } from "./PicEditorTools";
import PicEditorTools from "./PicEditorTools";
import {
  DisablePictureDrawPictureCommand,
  DisablePriorityDrawPictureCommand,
  SetPictureColorPictureCommand,
  SetPriorityColorPictureCommand,
} from "agikit-core/dist/Types/Picture";
import {
  CommandListNavigationContext,
  useCommandListNavigation,
} from "./CommandListNavigation";

export function PicEditor({
  pictureResource,
  setPictureResource,
}: {
  pictureResource: EditingPictureResource;
  setPictureResource: React.Dispatch<
    React.SetStateAction<EditingPictureResource>
  >;
}) {
  const [selectedTool, setSelectedTool] = useState<PictureTool>(
    PICTURE_TOOLS[0]
  );
  const [visualColor, setVisualColor] = useState<number | undefined>();
  const [priorityColor, setPriorityColor] = useState<number | undefined>();
  const [visualCursorPosition, setVisualCursorPosition] = useState<
    CursorPosition | undefined
  >();
  const [priorityCursorPosition, setPriorityCursorPosition] = useState<
    CursorPosition | undefined
  >();
  const renderedPicture = useMemo(
    () =>
      renderPicture({
        ...pictureResource,
        commands: pictureResource.commands.filter((command) => command.enabled),
      }),
    [pictureResource]
  );

  const setCommands = useCallback(
    (
      calculateNewState: (
        prevCommands: EditingPictureCommand[]
      ) => EditingPictureCommand[]
    ) =>
      setPictureResource((prevResource) => ({
        ...prevResource,
        commands: calculateNewState(prevResource.commands),
      })),
    [setPictureResource]
  );

  const navigationContextValue = useCommandListNavigation(
    pictureResource.commands,
    setCommands
  );
  const {
    enabledCommands,
    currentCommandId,
    jumpRelative,
  } = navigationContextValue;

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        event.stopPropagation();
        jumpRelative(1);
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        event.stopPropagation();
        jumpRelative(-1);
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [jumpRelative]);

  const currentCommandColors = useMemo(() => {
    const currentCommandIndex =
      currentCommandId != null
        ? enabledCommands.findIndex((c) => c.uuid === currentCommandId)
        : 0;
    const currentCommand = enabledCommands[currentCommandIndex];
    if (!currentCommand) {
      return { visual: undefined, priority: undefined };
    }

    const reversedCommandsThroughCurrent = enabledCommands
      .slice(0, currentCommandIndex + 1)
      .reverse();
    let visualCommand:
      | SetPictureColorPictureCommand
      | DisablePictureDrawPictureCommand
      | undefined;
    let priorityCommand:
      | SetPriorityColorPictureCommand
      | DisablePriorityDrawPictureCommand
      | undefined;
    for (let command of reversedCommandsThroughCurrent) {
      if (
        !visualCommand &&
        (command.type === "SetPictureColor" ||
          command.type === "DisablePictureDraw")
      ) {
        visualCommand = command;
      }

      if (
        !priorityCommand &&
        (command.type === "SetPriorityColor" ||
          command.type === "DisablePriorityDraw")
      ) {
        priorityCommand = command;
      }

      if (visualCommand && priorityCommand) {
        break;
      }
    }

    return {
      visual:
        visualCommand?.type === "SetPictureColor"
          ? visualCommand.colorNumber
          : undefined,
      priority:
        priorityCommand?.type === "SetPriorityColor"
          ? priorityCommand.colorNumber
          : undefined,
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
            buffer={renderedPicture.visualBuffer}
            onCursorMove={setVisualCursorPosition}
            onCursorDown={() => {}}
            onCursorOut={() => {
              setVisualCursorPosition(undefined);
            }}
          />
          <div>
            {visualCursorPosition ? (
              `Cursor position: ${visualCursorPosition.x}, ${visualCursorPosition.y}`
            ) : (
              <>&nbsp;</>
            )}
          </div>
        </div>

        <div className="pic-editor-priority-area">
          <h3>Priority</h3>
          <PicCanvas
            buffer={renderedPicture.priorityBuffer}
            onCursorMove={setPriorityCursorPosition}
            onCursorDown={() => {}}
            onCursorOut={() => {
              setPriorityCursorPosition(undefined);
            }}
          />
          <div>
            {priorityCursorPosition ? (
              `Cursor position: ${priorityCursorPosition.x}, ${priorityCursorPosition.y}`
            ) : (
              <>&nbsp;</>
            )}
          </div>
        </div>

        <div
          className="pic-editor-controls"
          style={{ display: "flex", flexDirection: "column" }}
        >
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
