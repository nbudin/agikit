import fs, { readFileSync } from 'fs';
import path from 'path';
import { agiLzwDecompress } from '..';
import { ResourceType, DirEntry, ResourceDir, Resource } from '../Types/Resources';

function readDirEntry(dirData: Buffer, index: number) {
  const volPlusHighOrderNybble = dirData.readUInt8(index);
  const midOrderByte = dirData.readUInt8(index + 1);
  const lowOrderByte = dirData.readUInt8(index + 2);

  const volumeNumber = volPlusHighOrderNybble >> 4;
  const offset = ((volPlusHighOrderNybble & 0x0f) << 16) + (midOrderByte << 8) + lowOrderByte;

  if (offset === 0xfffff && volumeNumber === 0x0f) {
    // 0xFFFFFF means the resource does not exist
    return undefined;
  } else {
    return { offset, volumeNumber };
  }
}

export function readDirData(dirData: Buffer, resourceType: ResourceType): (DirEntry | undefined)[] {
  const entries: (DirEntry | undefined)[] = [];
  let index = 0;
  let resourceNumber = 0;
  while (index < dirData.byteLength) {
    const entryData = readDirEntry(dirData, index);
    index += 3;

    if (entryData == null) {
      entries.push(undefined);
    } else {
      entries.push({
        ...entryData,
        resourceNumber,
        resourceType,
      });
    }

    resourceNumber += 1;
  }

  return entries;
}

export function readV2Dir(path: string, resourceType: ResourceType): (DirEntry | undefined)[] {
  const dirData = fs.readFileSync(path);
  return readDirData(dirData, resourceType);
}

const v2DirFiles = [
  ['LOGDIR', ResourceType.LOGIC],
  ['VIEWDIR', ResourceType.VIEW],
  ['PICDIR', ResourceType.PIC],
  ['SNDDIR', ResourceType.SOUND],
] as const;

export function readV2ResourceDirs(gamePath: string): ResourceDir {
  const resourceDir: Partial<ResourceDir> = {};

  v2DirFiles.forEach(([filename, resourceType]) => {
    resourceDir[resourceType] = readV2Dir(path.join(gamePath, filename), resourceType);
  });

  return resourceDir as ResourceDir;
}

export function readV3ResourceDir(gamePath: string, gameId: string): ResourceDir {
  const dirData = readFileSync(path.join(gamePath, `${gameId}DIR`));

  const logicStart = dirData.readUInt16LE(0);
  const picStart = dirData.readUInt16LE(2);
  const viewStart = dirData.readUInt16LE(4);
  const soundStart = dirData.readUInt16LE(6);

  return {
    LOGIC: readDirData(dirData.slice(logicStart, picStart), ResourceType.LOGIC),
    PIC: readDirData(dirData.slice(picStart, viewStart), ResourceType.PIC),
    VIEW: readDirData(dirData.slice(viewStart, soundStart), ResourceType.VIEW),
    SOUND: readDirData(dirData.slice(viewStart), ResourceType.SOUND),
  };
}

export function readV2Resource(gamePath: string, dirEntry: DirEntry): Resource {
  const volPath = path.join(gamePath, `VOL.${dirEntry.volumeNumber}`);
  const volData = fs.readFileSync(volPath);

  const signature = volData.readUInt16BE(dirEntry.offset);
  if (signature !== 0x1234) {
    throw new Error(`Invalid resource signature ${signature}`);
  }

  const resourceVolNumber = volData.readUInt8(dirEntry.offset + 2);
  if (resourceVolNumber !== dirEntry.volumeNumber) {
    throw new Error('Volume number mismatch');
  }

  const length = volData.readUInt16LE(dirEntry.offset + 3);
  const data = volData.slice(dirEntry.offset + 5, dirEntry.offset + 5 + length);
  return { type: dirEntry.resourceType, number: dirEntry.resourceNumber, data };
}

export function readV3Resource(gamePath: string, dirEntry: DirEntry, gameId: string): Resource {
  const volPath = path.join(gamePath, `${gameId}VOL.${dirEntry.volumeNumber}`);
  const volData = fs.readFileSync(volPath);

  const signature = volData.readUInt16BE(dirEntry.offset);
  if (signature !== 0x1234) {
    throw new Error(`Invalid resource signature ${signature}`);
  }

  const resourceVolNumberWithPicFlag = volData.readUInt8(dirEntry.offset + 2);
  const resourceVolNumber = resourceVolNumberWithPicFlag & 0b01111111;
  const isPic = (resourceVolNumberWithPicFlag & 0b10000000) > 0;
  if (resourceVolNumber !== dirEntry.volumeNumber) {
    throw new Error('Volume number mismatch');
  }

  const uncompressedLength = volData.readUInt16LE(dirEntry.offset + 3);
  const compressedLength = volData.readUInt16LE(dirEntry.offset + 5);

  const rawData = volData.slice(dirEntry.offset + 7, dirEntry.offset + compressedLength);
  const data =
    isPic || uncompressedLength === compressedLength ? rawData : agiLzwDecompress(rawData);

  return { type: dirEntry.resourceType, number: dirEntry.resourceNumber, data };
}
