import { useEffect, useMemo, useState } from "react";
import { PictureCommand } from "agikit-core/dist/Types/Picture";
import { buildPicture } from "agikit-core/dist/Build/BuildPicture";
import { Buffer } from "buffer";
import * as ReactDOM from "react-dom";
import { v4 as uuidv4 } from "uuid";
import { PicEditor } from "./PicEditor";
import { EditingPictureResource } from "./EditingPictureTypes";

import "./reset.css";
import "./vscode.css";
import "bootstrap-icons/font/bootstrap-icons.css";
import "./piceditor.css";
import {
  PicEditorControlContext,
  PicEditorControlContextValue,
} from "./PicEditorControlContext";

// @ts-expect-error
window.Buffer = Buffer;

// @ts-expect-error
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
  const [resolveConfirm, setResolveConfirm] = useState<
    (result: boolean) => void | undefined
  >();
  const controlContextValue = useMemo<PicEditorControlContextValue>(
    () => ({
      confirm: (message) => {
        if (resolveConfirm) {
          return Promise.reject("Already showing a confirmation quick pick");
        }
        return new Promise<boolean>((resolve) => {
          setResolveConfirm(() => {
            // we have to do it this way because this is the only way to set a function as a state
            // value
            return resolve;
          });
          vscode.postMessage({ type: "confirm", message });
        });
      },
      addCommands: (commands, afterCommandId) => {
        vscode.postMessge({ type: "addCommands", commands, afterCommandId });
      },
      deleteCommand: (commandId) => {
        vscode.postMessage({ type: "deleteCommand", commandId });
      },
    }),
    [resolveConfirm]
  );

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
            setPictureResource(body.resource);
            return;
          }
        }
        case "confirmResult":
          const result = body.confirmed;
          if (resolveConfirm) {
            resolveConfirm(result);
          }
          setResolveConfirm(undefined);
          return;
        case "update": {
          const newResource = body.content;
          setPictureResource(newResource);
          return;
        }
        case "getFileData": {
          vscode.postMessage({
            type: "response",
            requestId,
            body: Array.from(buildPicture(pictureResource)),
          });
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
    <PicEditorControlContext.Provider value={controlContextValue}>
      <PicEditor
        pictureResource={pictureResource}
        setPictureResource={setPictureResource}
      />
    </PicEditorControlContext.Provider>
  );
}

ReactDOM.render(
  <VscodePicEditor />,
  document.querySelector("#pic-editor-root")
);
