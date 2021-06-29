import { ObjectList, ObjectListEntry } from '../Types/ObjectList';
import { avisDurgan, xorBuffer } from '../XorEncryption';

export function readObjectList(objectData: Buffer): ObjectList {
  const decryptedData = xorBuffer(objectData, avisDurgan);
  const objectNamesOffset = decryptedData.readUInt16LE(0);
  const maxAnimatedObjects = decryptedData.readUInt8(2);
  const objects: ObjectListEntry[] = [];

  let headerOffset = 0;
  while (headerOffset < objectNamesOffset) {
    const nameOffset = decryptedData.readUInt16LE(3 + headerOffset);
    const startingRoomNumber = decryptedData.readUInt8(3 + headerOffset + 2);
    headerOffset += 3;

    const nameBytes: number[] = [];
    let nameByteOffset = nameOffset + 3;
    let nameByte: number;
    do {
      nameByte = decryptedData.readUInt8(nameByteOffset);
      if (nameByte !== 0) {
        nameBytes.push(nameByte);
        nameByteOffset += 1;
      }
    } while (nameByte !== 0);

    objects.push({
      name: Buffer.from(nameBytes).toString('ascii'),
      startingRoomNumber,
    });
  }

  return {
    maxAnimatedObjects,
    objects,
  };
}
