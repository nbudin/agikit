import { OperationReconSpeechBubbles } from '../../../testData';
import { agiLzwCompress, agiLzwDecompress } from '../LZW';
import { decodeBitstream } from './compressionTestUtils';

describe('LZW', () => {
  it('compresses a string to a known series of codes', () => {
    const original = Buffer.from('TOBEORNOTTOBEORTOBEORNOT', 'ascii');
    const compressed = agiLzwCompress(original);
    const codes = decodeBitstream(compressed, 9);

    expect(codes).toEqual([
      //   T   O   B   E   O   R   N   O   T   TO   BE   OR   TOB  EO   RN   OT   [eof]
      256, 84, 79, 66, 69, 79, 82, 78, 79, 84, 258, 260, 262, 267, 261, 263, 265, 257,
    ]);
  });

  it('compresses and decompresses a string', () => {
    const original = Buffer.from('TOBEORNOTTOBEORTOBEORNOT', 'ascii');
    const compressed = agiLzwCompress(original);
    const decompressed = agiLzwDecompress(compressed);
    expect(decompressed.toString('ascii')).toEqual(original.toString('ascii'));
  });

  it('compresses and decompresses a large resource', () => {
    const original = Buffer.concat([
      OperationReconSpeechBubbles,
      OperationReconSpeechBubbles,
      OperationReconSpeechBubbles,
      OperationReconSpeechBubbles,
      OperationReconSpeechBubbles,
    ]);
    const compressed = agiLzwCompress(original);
    const decompressed = agiLzwDecompress(compressed);
    expect(decompressed).toEqual(original);
  });
});
