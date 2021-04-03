import React from "react";
import { EGAPalette } from "agikit-core/dist/ColorPalettes";
import ColorSelector from "./ColorSelector";

export const PICTURE_TOOLS = [
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

export default function PicEditorTools({
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
