import { LZWBitstreamReader } from '../Bitstreams';
import { agiLzwCompress, agiLzwDecompress } from '../LZW';

const OperationReconBeep = Buffer.from('CAAPABEAEwACAAuOkP//////////', 'base64');

describe('LZW', () => {
  it('compresses a string to a known series of codes', () => {
    const original = Buffer.from('TOBEORNOTTOBEORTOBEORNOT', 'ascii');
    const compressed = agiLzwCompress(original);
    const reader = new LZWBitstreamReader(compressed);
    const codes: number[] = [];

    while (!reader.done()) {
      const code = reader.readCode(9);
      codes.push(code);
    }

    expect(codes).toEqual([
      84, 79, 66, 69, 79, 82, 78, 79, 84, 258, 260, 262, 267, 261, 263, 265, 257,
    ]);
  });

  it('compresses and decompresses a string', () => {
    const original = Buffer.from('TOBEORNOTTOBEORTOBEORNOT', 'ascii');
    const compressed = agiLzwCompress(original);
    const decompressed = agiLzwDecompress(compressed);
    expect(decompressed).toEqual(original);
  });
});
