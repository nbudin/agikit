import { flatMap } from 'lodash';
import { textEncryptionKey } from '../Extract/Logic/ReadLogic';

function xorEncrypt(cleartext: Buffer, key: Buffer): Buffer {
  const encrypted = Buffer.alloc(cleartext.byteLength);
  for (let offset = 0; offset < cleartext.byteLength; offset++) {
    const cleartextByte = cleartext.readUInt8(offset);
    const keyByte = key.readUInt8(offset % key.byteLength);
    encrypted.writeUInt8(cleartextByte ^ keyByte, offset);
  }
  return encrypted;
}

function encodeUInt16LE(number: number): number[] {
  const buffer = Buffer.alloc(2);
  buffer.writeUInt16LE(number);
  return [...buffer];
}

export function encodeMessages(messageArray: (string | undefined)[]): Buffer {
  const messageBuffers = messageArray.map((message) => {
    if (message == null) {
      return Buffer.alloc(0);
    }

    return Buffer.from(`${message}\0`, 'ascii');
  });

  const textSection = xorEncrypt(Buffer.concat(messageBuffers), textEncryptionKey);
  const messageHeaderLength = 3 + messageArray.length * 2;

  const messageOffsets: number[] = [];
  let offset = messageHeaderLength - 1;
  messageBuffers.forEach((buffer) => {
    messageOffsets.push(offset);
    offset += buffer.byteLength;
  });

  return Buffer.from([
    messageArray.length,
    ...encodeUInt16LE(messageHeaderLength + textSection.byteLength),
    ...flatMap(messageOffsets, encodeUInt16LE),
    ...textSection,
  ]);
}

export function encodeLogic(codeSection: Buffer, messageSection: Buffer): Buffer {
  return Buffer.concat([
    Buffer.from(encodeUInt16LE(codeSection.byteLength)),
    codeSection,
    messageSection,
  ]);
}
