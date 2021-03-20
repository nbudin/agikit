import React, {
  CSSProperties,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { renderPicture } from "agikit-core/dist/Extract/Picture/RenderPicture";
import { CursorPosition, PicCanvas } from "./PicCanvas";
import { EditingPictureResource } from "./EditingPictureTypes";
import { PicCommandList } from "./PicCommandList";
import { EGAPalette } from "agikit-core/dist/ColorPalettes";
import { usePopper } from "react-popper";
import useOnclickOutside from "react-cool-onclickoutside";

const PICTURE_TOOLS = [
  {
    name: "absoluteLine",
    description: "Absolute line",
    iconClass: "bi-pentagon",
  },
  {
    name: "relativeLine",
    description: "Relative line",
    iconClass: "bi-pencil",
  },
  { name: "corner", description: "Corner line", iconClass: "bi-bounding-box" },
  { name: "fill", description: "Fill", iconClass: "bi-paint-bucket" },
  { name: "pen", description: "Pen", iconClass: "bi-pen" },
] as const;

export type PictureTool = typeof PICTURE_TOOLS[number];

function backgroundStylesForColorNumber(
  colorNumber: number | undefined,
  palette: typeof EGAPalette
): CSSProperties {
  if (colorNumber != null) {
    return { backgroundColor: palette[colorNumber] };
  }

  return {
    background:
      "linear-gradient(45deg, rgba(0, 0, 0, 0.0980392) 25%, transparent 25%, transparent 75%, rgba(0, 0, 0, 0.0980392) 75%, rgba(0, 0, 0, 0.0980392) 0), linear-gradient(45deg, rgba(0, 0, 0, 0.0980392) 25%, transparent 25%, transparent 75%, rgba(0, 0, 0, 0.0980392) 75%, rgba(0, 0, 0, 0.0980392) 0), white",
    backgroundSize: "10px 10px, 10px 10px",
    backgroundPosition: "0 0, 5px 5px",
  };
}

function textColorForBackgroundColor(
  backgroundColor: number | undefined
): string {
  if (backgroundColor != null && backgroundColor < 10) {
    return "white";
  }

  return "black";
}

function stylesForColorNumber(
  colorNumber: number | undefined,
  palette: typeof EGAPalette
): CSSProperties {
  return {
    ...backgroundStylesForColorNumber(colorNumber, palette),
    color: textColorForBackgroundColor(colorNumber),
  };
}

function ColorSelector({
  palette,
  color,
  setColor,
  colorType,
}: {
  palette: typeof EGAPalette;
  color: number | undefined;
  setColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  colorType: string;
}) {
  const [button, setButton] = useState<HTMLButtonElement | null>(null);
  const [popover, setPopover] = useState<HTMLDivElement | null>(null);
  const [open, setOpen] = useState(false);
  const { styles, attributes } = usePopper(button, popover);
  const onClickOutsideRef = useOnclickOutside(() => {
    setOpen(false);
  });
  useEffect(() => {
    if (button) {
      onClickOutsideRef(button);
    }
    if (popover) {
      onClickOutsideRef(popover);
    }
  }, [onClickOutsideRef, button, popover]);

  return (
    <>
      <button
        ref={setButton}
        type="button"
        className={`secondary`}
        onClick={() => setOpen((prevOpen) => !prevOpen)}
        style={{
          width: "3rem",
          height: "3rem",
          margin: "1px",
          ...stylesForColorNumber(color, palette),
        }}
      >
        {colorType}: {color ?? "off"}
      </button>
      <div
        className="pic-editor-color-picker"
        style={{ ...styles.popper, visibility: open ? "visible" : "hidden" }}
        ref={setPopover}
        {...attributes.popper}
      >
        <div style={styles.arrow} {...attributes.arrow} />
        <div style={{ display: "flex", flexWrap: "wrap" }}>
          {palette.map((_colorValue, colorNumber) => (
            <button
              type="button"
              className="secondary"
              onClick={() => {
                setColor(colorNumber);
              }}
              style={{
                width: "3rem",
                height: "3rem",
                margin: "1px",
                ...stylesForColorNumber(colorNumber, palette),
              }}
            >
              {colorNumber}
            </button>
          ))}
          <button
            type="button"
            className="secondary"
            onClick={() => {
              setColor(undefined);
            }}
            style={{
              width: "3rem",
              height: "3rem",
              margin: "1px",
              ...stylesForColorNumber(undefined, palette),
            }}
          >
            off
          </button>
        </div>
      </div>
    </>
  );
}

function PicEditorTools({
  selectedTool,
  setSelectedTool,
  visualColor,
  setVisualColor,
  priorityColor,
  setPriorityColor,
}: {
  selectedTool: PictureTool;
  setSelectedTool: React.Dispatch<React.SetStateAction<PictureTool>>;
  visualColor: number | undefined;
  setVisualColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  priorityColor: number | undefined;
  setPriorityColor: React.Dispatch<React.SetStateAction<number | undefined>>;
}) {
  return (
    <div style={{ display: "flex" }}>
      {PICTURE_TOOLS.map((tool) => (
        <button
          key={tool.name}
          type="button"
          className={`secondary${selectedTool === tool ? " inverse" : ""}`}
          onClick={() => setSelectedTool(tool)}
          style={{
            width: "3rem",
            height: "3rem",
            fontSize: "1.5rem",
            margin: "1px",
          }}
          title={tool.description}
        >
          <i
            className={tool.iconClass}
            role="img"
            aria-label={tool.description}
          />
        </button>
      ))}
      <ColorSelector
        palette={EGAPalette}
        color={visualColor}
        setColor={setVisualColor}
        colorType="V"
      />
      <ColorSelector
        palette={EGAPalette}
        color={priorityColor}
        setColor={setPriorityColor}
        colorType="P"
      />
    </div>
  );
}

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

  return (
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
        <PicCommandList
          pictureResource={pictureResource}
          setPictureResource={setPictureResource}
        />
      </div>
    </div>
  );
}
