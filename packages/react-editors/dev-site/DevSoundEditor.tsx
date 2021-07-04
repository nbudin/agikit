import { readIBMPCjrSoundResource } from '@agikit/core/dist/Extract/Sound/ReadSound';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { operationReconThemeBase64 } from './dev-example-data';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
import '../styles/soundeditor.css';
import SoundEditor from '../src/SoundEditor';

const operationReconTheme = readIBMPCjrSoundResource(
  Buffer.from(operationReconThemeBase64, 'base64'),
);

function DevSoundEditor() {
  const sound = operationReconTheme;
  return <SoundEditor sound={sound} />;
}

window.addEventListener('load', () => {
  ReactDOM.render(<DevSoundEditor />, document.getElementById('sound-editor-root'));
});
