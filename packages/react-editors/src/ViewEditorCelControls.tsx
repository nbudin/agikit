import React, { useContext, useMemo, useState } from 'react';
import { EGAPalette } from '@agikit/core/dist/ColorPalettes';
import ColorSelector from './ColorSelector';
import { ViewEditorContext } from './ViewEditorContext';
import { ViewEditorControlContext } from './ViewEditorControlContext';
import { v4 } from 'uuid';

export function ViewEditorCelControls() {
  const {
    loopNumber,
    celNumber,
    currentLoop,
    currentCel,
    drawingColor,
    setDrawingColor,
  } = useContext(ViewEditorContext);
  const { addCommands, zoom, setZoom } = useContext(ViewEditorControlContext);
  const [editingSize, setEditingSize] = useState(false);
  const [newSize, setNewSize] = useState<{ width?: number; height?: number }>();

  const validNewSize = useMemo(() => {
    if (
      newSize &&
      newSize.width != null &&
      newSize.height != null &&
      newSize.width > 0 &&
      newSize.height > 0
    ) {
      return { width: newSize.width, height: newSize.height };
    }

    return undefined;
  }, [newSize]);

  const resizeCel = (newWidth: number, newHeight: number) => {
    if (!currentCel) {
      return;
    }

    addCommands([
      {
        type: 'Resize',
        width: newWidth,
        height: newHeight,
        loop: currentLoop?.type === 'mirrored' ? currentLoop.mirroredFromLoopNumber : loopNumber,
        cel: celNumber,
        originX: 0,
        originY: 0,
        uuid: v4(),
      },
    ]);
  };

  const sizeFieldChanged = (
    event: React.ChangeEvent<HTMLInputElement>,
    field: 'width' | 'height',
  ) => {
    let value: number | undefined = Number.parseInt(event.target.value, 10);
    if (Number.isNaN(value)) {
      value = undefined;
    }

    setNewSize((prevNewSize) => {
      if (!prevNewSize) {
        return;
      }

      return { ...prevNewSize, [field]: value };
    });
  };

  if (!currentLoop || !currentCel) {
    return <></>;
  }

  return (
    <div className="view-editor-cel-controls">
      <div>
        Loop {loopNumber}, cel {celNumber}
        {currentCel && (
          <>
            <br />
            {editingSize ? (
              <div className="view-editor-cel-size-editor">
                <input
                  type="number"
                  min={1}
                  value={newSize?.width}
                  onChange={(event) => sizeFieldChanged(event, 'width')}
                />
                x
                <input
                  type="number"
                  min={1}
                  value={newSize?.height}
                  onChange={(event) => sizeFieldChanged(event, 'height')}
                />
                <button
                  className="primary"
                  type="button"
                  disabled={validNewSize == null}
                  onClick={() => {
                    if (validNewSize) {
                      resizeCel(validNewSize.width, validNewSize.height);
                      setEditingSize(false);
                      setNewSize(undefined);
                    }
                  }}
                >
                  Resize
                </button>
                <button
                  className="secondary"
                  type="button"
                  onClick={() => {
                    setEditingSize(false);
                    setNewSize(undefined);
                  }}
                >
                  Cancel
                </button>
              </div>
            ) : (
              <>
                {currentCel.width}x{currentCel.height}
                <button
                  className="secondary"
                  type="button"
                  onClick={() => {
                    setEditingSize(true);
                    setNewSize({ width: currentCel.width, height: currentCel.height });
                  }}
                >
                  Edit
                </button>
                <br />
              </>
            )}
            Transparent color:{' '}
            <ColorSelector
              color={currentCel.transparentColor}
              palette={EGAPalette}
              hideOffOption
              setColor={(newColor) => {
                if (newColor != null) {
                  addCommands([
                    {
                      type: 'ChangeTransparentColor',
                      loop:
                        currentLoop.type === 'mirrored'
                          ? currentLoop.mirroredFromLoopNumber
                          : loopNumber,
                      cel: celNumber,
                      transparentColor: newColor,
                      uuid: v4(),
                    },
                  ]);
                }
              }}
            />
            <br />
            {currentLoop.type === 'mirrored' ? (
              <>Mirrored from loop {currentLoop.mirroredFromLoopNumber}</>
            ) : (
              <>
                {currentLoop.mirroredByLoopNumbers.length > 0 ? (
                  <>
                    Mirrored by {currentLoop.mirroredByLoopNumbers.length === 1 ? 'loop' : 'loops'}{' '}
                    {currentLoop.mirroredByLoopNumbers.join(', ')}
                  </>
                ) : (
                  <>Not mirrored by any other loops</>
                )}
              </>
            )}
          </>
        )}
      </div>
      <div className="view-editor-tools">
        <button
          type="button"
          className="agikit-tool-button secondary"
          title="Zoom out"
          disabled={zoom <= 1}
          onClick={() => {
            if (zoom > 1) {
              setZoom(zoom - 1);
            }
          }}
        >
          <i className="bi-zoom-out" role="img" aria-label="Zoom out" />
        </button>

        <button
          type="button"
          className="agikit-tool-button secondary"
          title="Zoom in"
          onClick={() => setZoom(zoom + 1)}
        >
          <i className="bi-zoom-in" role="img" aria-label="Zoom in" />
        </button>

        <ColorSelector
          palette={EGAPalette}
          color={drawingColor}
          setColor={setDrawingColor}
          transparentColor={currentCel.transparentColor}
        />
      </div>
    </div>
  );
}
