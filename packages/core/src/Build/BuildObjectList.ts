import { flatMap } from 'lodash';
import { getXorEncryptionKey, xorBuffer } from '../XorEncryption';
import { ObjectList } from '../Types/ObjectList';
import { encodeUInt16LE } from '../DataEncoding';

export function buildObjectList(objectList: ObjectList): Buffer {
  const objectNames = objectList.objects.map((object) => Buffer.from(`${object.name}\0`, 'ascii'));
  const headerLength = objectList.objects.length * 3;
  let objectOffset = 0;
  const objectHeaders = flatMap(objectList.objects, (object, index) => {
    const thisObjectOffset = objectOffset;
    objectOffset += objectNames[index].byteLength;
    return [...encodeUInt16LE(thisObjectOffset + headerLength), object.startingRoomNumber];
  });
  const cleartextObjectList = Buffer.concat([
    Buffer.from([...encodeUInt16LE(headerLength), objectList.maxAnimatedObjects, ...objectHeaders]),
    ...objectNames,
  ]);
  return xorBuffer(cleartextObjectList, getXorEncryptionKey());
}
