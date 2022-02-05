import { getXorEncryptionKey, xorBuffer } from '../../XorEncryption';
import { AGIVersion } from '../../Types/AGIVersion';
import { LogicProgram } from '../../Types/Logic';
import { readInstructions } from './LogicDisasm';

function readMessages(textData: Buffer): (string | undefined)[] {
  const messageCount = textData.readUInt8(0);
  // why do they even have this in the format?
  // const endOfMessages = textData.readUInt16LE(1);
  const messageHeaderLength = 3 + messageCount * 2;
  const messages: (string | undefined)[] = [];
  const decryptedMessageSection = xorBuffer(
    textData.slice(messageHeaderLength),
    getXorEncryptionKey(),
  );
  const decryptedTextData = Buffer.concat([
    textData.slice(0, messageHeaderLength),
    decryptedMessageSection,
  ]);
  for (let messageIndex = 0; messageIndex < messageCount; messageIndex++) {
    const messageOffset = decryptedTextData.readUInt16LE(3 + messageIndex * 2) + 1;
    const messageBytes: number[] = [];
    let byteOffset = 0;

    if (messageOffset === 1) {
      // there was a 0x0000 in the header (we add 1 to it in messageOffset)
      messages.push(undefined);
      continue;
    }

    while (messageOffset + byteOffset < decryptedTextData.byteLength) {
      const messageByte = decryptedTextData.readUInt8(messageOffset + byteOffset);
      if (messageByte === 0) {
        const message = Buffer.from(messageBytes).toString('ascii');
        messages.push(message);
        break;
      }

      messageBytes.push(messageByte);
      byteOffset += 1;
    }
  }

  return messages;
}

export function readLogicResource(resourceData: Buffer, agiVersion: AGIVersion): LogicProgram {
  const textOffset = resourceData.readUInt16LE(0);
  const codeData = resourceData.slice(2, textOffset + 2);
  const textData = resourceData.slice(textOffset + 2);

  return {
    instructions: readInstructions(codeData, agiVersion),
    messages: readMessages(textData),
  };
}
