import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { readViewResource } from 'agikit-core/dist/Extract/View/ReadView';
import { ViewEditor } from '../src/ViewEditor';
import { templateEgoBase64 } from './dev-example-data';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
import '../styles/vieweditor.css';

// @ts-expect-error
window.Buffer = Buffer;

const templateEgo = readViewResource(Buffer.from(templateEgoBase64, 'base64'));

const DevViewEditor = () => {
  return <ViewEditor view={templateEgo} />;
};

window.addEventListener('load', () => {
  ReactDOM.render(<DevViewEditor />, document.getElementById('view-editor-root'));
});
