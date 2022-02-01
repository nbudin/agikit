import { readIBMPCjrSoundResource } from '@agikit/core';
import ReactDOM from 'react-dom';
import { Buffer } from 'buffer';
import { operationReconThemeBase64 } from './dev-example-data';

import 'bootstrap-icons/font/bootstrap-icons.css';
import './dev-site.css';
import '../styles/common.css';
import '../styles/soundeditor.css';
import SoundEditor from '../src/SoundEditor';
import { sq2NoiseEffectBase64 } from './dev-example-data-do-not-check-in';

const operationReconTheme = readIBMPCjrSoundResource(
  Buffer.from(operationReconThemeBase64, 'base64'),
);

const sq2NoiseEffect = readIBMPCjrSoundResource(Buffer.from(sq2NoiseEffectBase64, 'base64'));

function DevSoundEditor() {
  return <SoundEditor sound={operationReconTheme} />;
}

window.addEventListener('load', () => {
  ReactDOM.render(<DevSoundEditor />, document.getElementById('sound-editor-root'));
});
