import React from 'react';
import { EGAPalette } from '@agikit/core/dist/ColorPalettes';
import ColorSelector from './ColorSelector';
import { PictureCommand, PicturePenSettings } from '@agikit/core/dist/Types/Picture';
import PenSettingsSelector from './PenSettingsSelector';

export const PICTURE_TOOLS = [
  {
    name: 'absoluteLine',
    description: 'Absolute line',
    iconClass: 'bi-pentagon',
  },
  {
    name: 'relativeLine',
    description: 'Relative line',
    iconClass: 'bi-pencil',
  },
  { name: 'corner', description: 'Corner line', iconClass: 'bi-bounding-box' },
  { name: 'fill', description: 'Fill', iconClass: 'bi-paint-bucket' },
  { name: 'pen', description: 'Pen', iconClass: 'bi-pen' },
] as const;

export type PictureTool = typeof PICTURE_TOOLS[number];

export default function PicEditorTools({
  commandInProgress,
  commitCommandInProgress,
  cancelCommandInProgress,
  selectedTool,
  setSelectedTool,
  visualColor,
  setVisualColor,
  priorityColor,
  setPriorityColor,
  penSettings,
  setPenSettings,
}: {
  commandInProgress: PictureCommand | undefined;
  commitCommandInProgress: () => void;
  cancelCommandInProgress: () => void;
  selectedTool: PictureTool;
  setSelectedTool: React.Dispatch<React.SetStateAction<PictureTool>>;
  visualColor: number | undefined;
  setVisualColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  priorityColor: number | undefined;
  setPriorityColor: React.Dispatch<React.SetStateAction<number | undefined>>;
  penSettings: PicturePenSettings;
  setPenSettings: React.Dispatch<React.SetStateAction<PicturePenSettings>>;
}) {
  return (
    <div style={{ display: 'flex' }}>
      {commandInProgress ? (
        <>
          <button
            type="button"
            className="agikit-tool-button primary"
            title="Finish command"
            onClick={commitCommandInProgress}
          >
            <i className="bi-check" role="img" aria-label="Finish command" />
          </button>
          <button
            type="button"
            className="agikit-tool-button secondary"
            title="Cancel command"
            onClick={cancelCommandInProgress}
          >
            <i className="bi-x" role="img" aria-label="Cancel command" />
          </button>
        </>
      ) : (
        PICTURE_TOOLS.map((tool) => (
          <button
            key={tool.name}
            type="button"
            className={`agikit-tool-button secondary${selectedTool === tool ? ' inverse' : ''}`}
            onClick={() => setSelectedTool(tool)}
            title={tool.description}
          >
            <i className={tool.iconClass} role="img" aria-label={tool.description} />
          </button>
        ))
      )}
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
      <PenSettingsSelector penSettings={penSettings} setPenSettings={setPenSettings} />
    </div>
  );
}
