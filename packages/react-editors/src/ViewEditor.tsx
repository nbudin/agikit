import React, { useMemo, useState } from 'react';
import { AGIView } from 'agikit-core/dist/Types/View';

import { ViewLoopEditor } from './ViewLoopEditor';
import { buildEditingView, EditingView } from './EditingViewTypes';
import ColorSelector from './ColorSelector';

export function ViewEditor({ view }: { view: AGIView }) {
  const editingView: EditingView = useMemo(() => buildEditingView(view), [view]);
  const [currentLoopNumber, setCurrentLoopNumber] = useState(0);
  const [currentCelNumber, setCurrentCelNumber] = useState(0);
  const [zoom, setZoom] = useState(6);

  const currentLoop = view.loops[currentLoopNumber];

  return (
    <div className="view-editor">
      <ViewLoopEditor
        view={editingView}
        loopNumber={currentLoopNumber}
        celNumber={currentCelNumber}
        setCelNumber={setCurrentCelNumber}
        zoom={zoom}
        setZoom={setZoom}
        key={currentLoopNumber}
      />
      <div className="view-editor-navigation-controls">
        <h2>Loops</h2>
        <ul className="view-editor-loop-list">
          {editingView.loops.map((loop, loopNumber) => (
            <li key={loopNumber}>
              <button
                type="button"
                className={loopNumber === currentLoopNumber ? 'current' : undefined}
                onClick={() => {
                  setCurrentLoopNumber(loopNumber);
                  if (currentCelNumber >= view.loops[loopNumber].cels.length) {
                    setCurrentCelNumber(0);
                  }
                }}
              >
                Loop {loopNumber}
                {loop.type === 'mirrored' &&
                  ` (mirrors loop ${editingView.loops.indexOf(loop.mirroredFromLoop)})`}
              </button>
            </li>
          ))}
        </ul>

        <h2>Cels</h2>
        <ul className="view-editor-cel-list">
          {(currentLoop?.cels ?? []).map((cel, celNumber) => (
            <li key={celNumber}>
              <button
                type="button"
                className={celNumber === currentCelNumber ? 'current' : undefined}
                onClick={() => {
                  setCurrentCelNumber(celNumber);
                }}
              >
                Cel {celNumber}
              </button>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
