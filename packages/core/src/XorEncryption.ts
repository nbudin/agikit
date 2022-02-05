export function getXorEncryptionKey(): Buffer {
  return Buffer.from('Avis Durgan', 'ascii');
}

export function xorBuffer(input: Buffer, encryptionKey: Buffer): Buffer {
  const output = Buffer.alloc(input.byteLength);
  let byteOffset = 0;
  while (byteOffset < input.byteLength) {
    const keyIndex = byteOffset % encryptionKey.byteLength;
    const keyByte = encryptionKey.readUInt8(keyIndex);
    const inputByte = input.readUInt8(byteOffset);
    const outputByte = inputByte ^ keyByte;
    output.writeUInt8(outputByte, byteOffset);
    byteOffset += 1;
  }

  return output;
}
