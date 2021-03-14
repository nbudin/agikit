import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import {
  PictureCommand,
  PictureCoordinate,
  PictureResource,
} from "agikit-core/dist/Types/Picture";
import { renderPicture } from "agikit-core/dist/Extract/Picture/RenderPicture";
import { EGAPalette } from "agikit-core/dist/ColorPalettes";
import { Buffer } from "buffer";
import * as ReactDOM from "react-dom";
import { v4 as uuidv4 } from "uuid";
import React = require("react");

// @ts-expect-error
window.Buffer = Buffer;

// @ts-ignore
const vscode = acquireVsCodeApi();
vscode.postMessage({ type: "ready" });

type EditingPictureCommand = PictureCommand & {
  uuid: string;
  enabled: boolean;
};

type EditingPictureResource = Omit<PictureResource, "commands"> & {
  commands: EditingPictureCommand[];
};

function PicCanvas({ buffer }: { buffer: Uint8Array }) {
  const ref = useRef<HTMLCanvasElement>(null);
  useLayoutEffect(() => {
    if (!ref.current) {
      return;
    }

    const ctx = ref.current.getContext("2d");
    if (!ctx) {
      return;
    }

    for (let index = 0; index < buffer.length; index++) {
      const x = index % 160;
      const y = Math.floor(index / 160);
      const color = EGAPalette[buffer[index]];
      ctx.fillStyle = color;
      ctx.fillRect(x * 4, y * 2, 4, 2);
    }
  }, [buffer]);

  return <canvas width="640" height="400" ref={ref}></canvas>;
}

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
        {command.type} ${command.colorNumber}
      </span>
    );
  }

  return command.type;
}

function PicEditor({
  pictureResource,
  setPictureResource,
}: {
  pictureResource: EditingPictureResource;
  setPictureResource: React.Dispatch<
    React.SetStateAction<EditingPictureResource>
  >;
}) {
  const renderedPicture = useMemo(
    () =>
      renderPicture({
        ...pictureResource,
        commands: pictureResource.commands.filter((command) => command.enabled),
      }),
    [pictureResource]
  );

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

  const setAllCommandsEnabled = (enabled: boolean) => {
    setPictureResource((prevResource) => ({
      ...prevResource,
      commands: prevResource.commands.map((command) => ({
        ...command,
        enabled,
      })),
    }));
  };

  const disableAllAfter = (uuid: string) => {
    setPictureResource((prevResource) => {
      const disableAfterIndex = prevResource.commands.findIndex(
        (command) => command.uuid === uuid
      );
      return {
        ...prevResource,
        commands: prevResource.commands.map((command, index) => ({
          ...command,
          enabled: index > disableAfterIndex ? false : command.enabled,
        })),
      };
    });
  };

  return (
    <div style={{ display: "flex" }}>
      <div>
        <PicCanvas buffer={renderedPicture.visualBuffer} />
        <PicCanvas buffer={renderedPicture.priorityBuffer} />
      </div>

      <div
        style={{ display: "flex", flexDirection: "column", height: "800px" }}
      >
        <div style={{ display: "flex" }}>
          <button type="button" onClick={() => setAllCommandsEnabled(true)}>
            Enable all
          </button>
          <button type="button" onClick={() => setAllCommandsEnabled(false)}>
            Disable all
          </button>
        </div>
        <div style={{ overflowY: "scroll" }}>
          <ul>
            {pictureResource.commands.map((command) => (
              <li key={command.uuid}>
                <div style={{ display: "flex" }}>
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
                    onClick={() => disableAllAfter(command.uuid)}
                    style={{ width: "auto" }}
                  >
                    Isolate
                  </button>
                </div>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}

function VscodePicEditor() {
  const [
    pictureResource,
    setPictureResource,
  ] = useState<EditingPictureResource>({
    commands: [],
  });
  const [editable, setEditable] = useState(false);

  useEffect(() => {
    const messageHandler = async (e: MessageEvent) => {
      const { type, body, requestId } = e.data;
      switch (type) {
        case "init": {
          setEditable(body.editable);
          if (body.untitled) {
            setPictureResource({ commands: [] });
            return;
          } else {
            // Load the initial image into the canvas.
            setPictureResource({
              ...body.resource,
              commands: body.resource.commands.map(
                (command: PictureCommand) => ({
                  ...command,
                  uuid: uuidv4(),
                  enabled: true,
                })
              ),
            });
            return;
          }
        }
        case "update": {
          const data = body.content
            ? new Uint8Array(body.content.data)
            : undefined;
          // const strokes = body.edits.map(
          //   (edit) => new Stroke(edit.color, edit.stroke)
          // );
          // await editor.reset(data, strokes);
          return;
        }
        case "getFileData": {
          // Get the image data for the canvas and post it back to the extension.
          // editor.getImageData().then((data) => {
          //   vscode.postMessage({
          //     type: "response",
          //     requestId,
          //     body: Array.from(data),
          //   });
          // });
          return;
        }
      }
    };
    window.addEventListener("message", messageHandler);

    return () => {
      window.removeEventListener("message", messageHandler);
    };
  });

  return (
    <PicEditor
      pictureResource={pictureResource}
      setPictureResource={setPictureResource}
    />
  );
}

ReactDOM.render(
  <VscodePicEditor />,
  document.querySelector("#pic-editor-root")
);
