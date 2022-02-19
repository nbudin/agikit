import { LZWBitstreamReader } from '../Bitstreams';

export function showBufferAsBinary(buffer: Buffer) {
  const bytes: string[] = [];

  for (const byte of [...buffer]) {
    bytes.push(byte.toString(2).padStart(8, '0'));
  }

  console.log(bytes.join(' '));
}

export function decodeBitstream(bitstream: Buffer, codeLength: number): number[] {
  const reader = new LZWBitstreamReader(bitstream);
  const codes: number[] = [];

  while (!reader.done()) {
    const code = reader.readCode(codeLength);
    codes.push(code);
  }

  return codes;
}
