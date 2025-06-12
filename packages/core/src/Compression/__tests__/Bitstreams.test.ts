import { PicBitstreamReader, PicBitstreamWriter } from '../Bitstreams';

describe('Pic bitstreams', () => {
  it('writes and reads back some codes correctly', () => {
    const codes = [1, 2, 3, 4];
    const writer = new PicBitstreamWriter();

    for (const code of codes) {
      writer.writeCode(code, 4);
    }

    const encoded = writer.finish();
    const reader = new PicBitstreamReader(encoded);
    const readCodes: number[] = [];

    while (!reader.done()) {
      readCodes.push(reader.readCode(4));
    }

    expect(readCodes).toEqual(codes);
  });
});
