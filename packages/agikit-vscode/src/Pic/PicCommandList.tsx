import { EGAPalette } from "agikit-core/dist/ColorPalettes";
import {
  PictureCommand,
  PictureCoordinate,
} from "agikit-core/dist/Types/Picture";
import React from "react";
import { EditingPictureResource } from "./EditingPictureTypes";

function describePoint(point: PictureCoordinate) {
  return `(${point.x}, ${point.y})`;
}

function describeCommand(command: PictureCommand): React.ReactNode {
  if (command.type === "AbsoluteLine") {
    return `Absolute line with points: ${command.points
      .map(describePoint)
      .join(", ")}`;
  }

  if (command.type === "RelativeLine") {
    return `Relative line from ${describePoint(
      command.startPosition
    )} with points: ${command.relativePoints.map(describePoint).join(", ")}`;
  }

  if (command.type === "DrawXCorner" || command.type === "DrawYCorner") {
    return `${command.type} from ${describePoint(
      command.startPosition
    )} with steps: ${command.steps
      .map((step) => `${step.axis} to ${step.position}`)
      .join(", ")}`;
  }

  if (command.type === "PlotWithPen") {
    return `Plot with pen at points: ${command.points
      .map(
        (plotPoint) =>
          `${describePoint(plotPoint.position)}${
            plotPoint.texture == null ? "" : `[${plotPoint.texture}]`
          }`
      )
      .join(", ")}`;
  }

  if (command.type === "Fill") {
    return `Fill at points: ${command.startPositions
      .map(describePoint)
      .join(", ")}`;
  }

  if (command.type === "ChangePen") {
    return `Change pen to ${JSON.stringify(command.settings)}`;
  }

  if (
    command.type === "SetPictureColor" ||
    command.type === "SetPriorityColor"
  ) {
    return (
      <span
        style={{
          color: EGAPalette[command.colorNumber],
          backgroundColor: command.colorNumber < 10 ? "white" : "black",
        }}
      >
        {command.type} {command.colorNumber}
      </span>
    );
  }

  return command.type;
}

export function PicCommandList({
  pictureResource,
  setPictureResource,
}: {
  pictureResource: EditingPictureResource;
  setPictureResource: React.Dispatch<
    React.SetStateAction<EditingPictureResource>
  >;
}) {
  const setAllCommandsEnabled = (enabled: boolean) => {
    setPictureResource((prevResource) => ({
      ...prevResource,
      commands: prevResource.commands.map((command) => ({
        ...command,
        enabled,
      })),
    }));
  };

  const setCommandEnabled = (uuid: string, enabled: boolean) => {
    setPictureResource((prevResource) => ({
      ...prevResource,
      commands: prevResource.commands.map((command) => {
        if (command.uuid === uuid) {
          return { ...command, enabled };
        }

        return command;
      }),
    }));
  };

  const jumpTo = (uuid: string) => {
    setPictureResource((prevResource) => {
      const disableAfterIndex = prevResource.commands.findIndex(
        (command) => command.uuid === uuid
      );
      return {
        ...prevResource,
        commands: prevResource.commands.map((command, index) => ({
          ...command,
          enabled: index > disableAfterIndex ? false : true,
        })),
      };
    });
  };

  return (
    <div
      className="pic-editor-controls"
      style={{ display: "flex", flexDirection: "column" }}
    >
      <div style={{ display: "flex" }}>
        <button
          type="button"
          style={{ margin: "1px" }}
          onClick={() => setAllCommandsEnabled(true)}
        >
          Enable all
        </button>
        <button
          type="button"
          style={{ margin: "1px" }}
          onClick={() => setAllCommandsEnabled(false)}
        >
          Disable all
        </button>
      </div>
      <div style={{ overflowY: "scroll" }}>
        <ul className="pic-editor-command-list">
          {pictureResource.commands.map((command) => (
            <li key={command.uuid}>
              <div style={{ display: "flex", alignItems: "center" }}>
                <input
                  type="checkbox"
                  value={command.uuid}
                  checked={command.enabled}
                  onChange={(event) =>
                    setCommandEnabled(command.uuid, event.target.checked)
                  }
                />
                <div style={{ flexGrow: 1 }}>{describeCommand(command)}</div>
                <button
                  type="button"
                  onClick={() => jumpTo(command.uuid)}
                  className="secondary pic-editor-go-to-command-button"
                >
                  Go to
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
