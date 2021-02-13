import fs from 'fs';
import path from 'path';
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

export function readV2Dir(path: string, resourceType: ResourceType): (DirEntry | undefined)[] {
  const dirData = fs.readFileSync(path);
  const entries: (DirEntry | undefined)[] = [];
  let index = 0;
  let resourceNumber = 0;
  while (index < dirData.byteLength) {
    const entryData = readDirEntry(dirData, index);
    index += 3;

    if (entryData == null) {
      // 0xFFFFFF means the resource does not exist
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
