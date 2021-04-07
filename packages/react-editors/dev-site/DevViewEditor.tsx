import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { readViewResource } from 'agikit-core/dist/Extract/View/ReadView';
import { templateEgoBase64 } from './dev-example-data';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/vieweditor.css';
import { ViewLoopEditor } from '../src/ViewLoopEditor';

// @ts-expect-error
window.Buffer = Buffer;

const templateEgo = readViewResource(Buffer.from(templateEgoBase64, 'base64'));

const DevViewEditor = () => {
  return (
    <>
      <h2>Loops</h2>
      <ul>
        {templateEgo.loops.map((loop, loopNumber) => (
          <li key={loopNumber}>
            <ViewLoopEditor view={templateEgo} loopNumber={loopNumber} />
          </li>
        ))}
      </ul>
    </>
  );
};

window.addEventListener('load', () => {
  ReactDOM.render(<DevViewEditor />, document.getElementById('view-editor-root'));
});
