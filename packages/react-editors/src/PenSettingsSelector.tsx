import React from 'react';
import { stylesForColorNumber } from './colorUtils';
import { usePopoverButton } from './usePopoverButton';
import { PicturePenSettings } from '@agikit/core/dist/Types/Picture';

export default function PenSettingsSelector({
  penSettings,
  setPenSettings,
}: {
  penSettings: PicturePenSettings;
  setPenSettings: React.Dispatch<React.SetStateAction<PicturePenSettings>>;
}) {
  const { setButton, setPopover, styles, attributes, open, setOpen } = usePopoverButton();

  return (
    <>
      <button
        ref={setButton}
        type="button"
        className="pic-editor-popover-button secondary"
        onClick={() => setOpen((prevOpen) => !prevOpen)}
      >
        {penSettings.shape === 'rectangle' ? 'R' : 'C'}
        {penSettings.splatter ? 'splat' : ''} {penSettings.size}
      </button>
      <div
        className="pic-editor-color-picker"
        style={{ ...styles.popper, visibility: open ? 'visible' : 'hidden' }}
        ref={setPopover}
        {...attributes.popper}
      >
        <div style={styles.arrow} {...attributes.arrow} />
        <select
          value={penSettings.shape}
          onChange={(event) =>
            setPenSettings((prevPenSettings) => ({
              ...prevPenSettings,
              shape: event.target.value as PicturePenSettings['shape'],
            }))
          }
        >
          <option value="rectangle">Rectangle</option>
          <option value="circle">Circle</option>
        </select>
        <select
          value={penSettings.size}
          onChange={(event) =>
            setPenSettings((prevPenSettings) => ({
              ...prevPenSettings,
              size: Number.parseInt(event.target.value, 10),
            }))
          }
        >
          {Array.from({ length: 8 }).map((_v, index) => (
            <option key={index} value={index}>
              Size {index}
            </option>
          ))}
        </select>
        <select
          value={penSettings.splatter ? 'true' : 'false'}
          onChange={(event) =>
            setPenSettings((prevPenSettings) => ({
              ...prevPenSettings,
              splatter: event.target.value === 'true',
            }))
          }
        >
          <option value="true">Splatter</option>
          <option value="false">Solid</option>
        </select>
      </div>
    </>
  );
}
