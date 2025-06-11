import fs, { readFileSync } from 'fs';
import path from 'path';
import { agiLzwDecompress, DirEntry, readDirData, readV2Dir, ResourceType } from '..';
import { ResourceDir, Resource } from '../Types/Resources';

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
    SOUND: readDirData(dirData.slice(soundStart), ResourceType.SOUND),
  };
}
