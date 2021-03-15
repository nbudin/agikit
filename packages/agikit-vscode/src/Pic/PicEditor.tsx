import React, { useMemo } from "react";
import { renderPicture } from "agikit-core/dist/Extract/Picture/RenderPicture";
import { PicCanvas } from "./PicCanvas";
import { EditingPictureResource } from "./EditingPictureTypes";
import { PicCommandList } from "./PicCommandList";

export function PicEditor({
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

  return (
    <div className="pic-editor">
      <div className="pic-editor-visual-area">
        <h3>Visual</h3>
        <PicCanvas buffer={renderedPicture.visualBuffer} />
      </div>

      <div className="pic-editor-priority-area">
        <h3>Priority</h3>
        <PicCanvas buffer={renderedPicture.priorityBuffer} />
      </div>

      <PicCommandList
        pictureResource={pictureResource}
        setPictureResource={setPictureResource}
      />
    </div>
  );
}
