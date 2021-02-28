export function encodeUInt16LE(int: number): number[] {
  return [int & 0xff, (int & 0xff00) >> 8];
}
