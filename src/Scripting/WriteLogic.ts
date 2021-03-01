import { flatMap } from 'lodash';
import { encodeUInt16LE } from '../DataEncoding';
import { avisDurgan, xorBuffer } from '../XorEncryption';

export function encodeMessages(messageArray: (string | undefined)[]): Buffer {
  const messageBuffers: Buffer[] = [];

  for (let index = 0; index < messageArray.length; index++) {
    const message = messageArray[index];
    if (message == null) {
      messageBuffers.push(Buffer.alloc(0));
    } else {
      messageBuffers.push(Buffer.from(`${message}\0`, 'ascii'));
    }
  }

  const textSection = xorBuffer(Buffer.concat(messageBuffers), avisDurgan);
  const messageHeaderLength = 3 + messageArray.length * 2;

  const messageOffsets: number[] = [];
  let offset = messageHeaderLength - 1;
  messageBuffers.forEach((buffer) => {
    messageOffsets.push(buffer.byteLength > 0 ? offset : 0);
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
