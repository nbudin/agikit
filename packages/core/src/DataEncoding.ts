export function encodeUInt16LE(int: number): number[] {
  return [int & 0xff, (int & 0xff00) >> 8];
}

export function encodeUInt16BE(int: number): number[] {
  return [(int & 0xff00) >> 8, int & 0xff];
}
