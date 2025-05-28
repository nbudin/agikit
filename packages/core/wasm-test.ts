import { readViewResource } from './pkg/agikit_core';
import { readFileSync } from 'fs';

const view = readViewResource(readFileSync('./test_data/1.agiview'));
console.log(view.loops.map((loop) => loop.cels.length));
