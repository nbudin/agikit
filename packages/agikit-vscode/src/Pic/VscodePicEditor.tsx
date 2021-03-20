import { useEffect, useState } from "react";
import { PictureCommand } from "agikit-core/dist/Types/Picture";
import { Buffer } from "buffer";
import * as ReactDOM from "react-dom";
import { v4 as uuidv4 } from "uuid";
import { PicEditor } from "./PicEditor";
import { EditingPictureResource } from "./EditingPictureTypes";

import "./reset.css";
import "./vscode.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import "./piceditor.css";

// @ts-expect-error
window.Buffer = Buffer;

// @ts-ignore
const vscode = acquireVsCodeApi();
vscode.postMessage({ type: "ready" });

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
