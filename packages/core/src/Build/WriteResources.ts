import fs from 'fs';
import path from 'path';
import { DirEntry, Resource, ResourceDir, ResourceType } from '../Types/Resources';
import filesize from 'filesize';
import { agiLzwCompress, Logger } from '..';

// const MAX_VOLUME_SIZE = 0xfffff;
const MAX_VOLUME_SIZE = 144 * 1024; // for testing purposes

type EncodedResource = Resource & {
  encodedData: Buffer;
};

export function encodeV2Resource(volumeNumber: number, resource: Resource): EncodedResource {
  const resourceHeader = Buffer.alloc(5);
  resourceHeader.writeUInt16BE(0x1234, 0);
  resourceHeader.writeUInt8(volumeNumber, 2);
  resourceHeader.writeUInt16LE(resource.data.byteLength, 3);

  return { ...resource, encodedData: Buffer.concat([resourceHeader, resource.data]) };
}

export function encodeV3Resource(volumeNumber: number, resource: Resource): EncodedResource {
  const compressedData = agiLzwCompress(resource.data);
  const storeCompressed =
    resource.type !== ResourceType.PIC && compressedData.byteLength < resource.data.byteLength;

  const resourceHeader = Buffer.alloc(7);
  resourceHeader.writeUInt16BE(0x1234, 0);
  // high-order bit of the volume number is used to flag whether or not it's a PIC
  resourceHeader.writeUInt8(volumeNumber | (resource.type === ResourceType.PIC ? 0x80 : 0), 2);
  resourceHeader.writeUInt16LE(resource.data.byteLength, 3);
  resourceHeader.writeUInt16LE(
    storeCompressed ? compressedData.byteLength : resource.data.byteLength,
    5,
  );

  return {
    ...resource,
    encodedData: Buffer.concat([resourceHeader, storeCompressed ? compressedData : resource.data]),
  };
}

export function writeVolume(
  volumeNumber: number,
  resources: EncodedResource[],
): [Buffer, DirEntry[]] {
  let offset = 0;
  const dirEntries: DirEntry[] = [];
  const volumeData: Buffer[] = [];

  resources.forEach((encodedResource) => {
    volumeData.push(encodedResource.encodedData);
    dirEntries.push({
      offset: offset,
      resourceNumber: encodedResource.number,
      resourceType: encodedResource.type,
      volumeNumber,
    });
    offset += encodedResource.encodedData.byteLength;

    if (offset > MAX_VOLUME_SIZE) {
      throw new Error('Resources too big to fit in a volume');
    }
  });

  return [Buffer.concat(volumeData), dirEntries];
}

// https://stackoverflow.com/a/53187807
export function findLastIndex<T>(
  array: Array<T>,
  predicate: (value: T, index: number, obj: T[]) => boolean,
): number {
  let l = array.length;
  while (l--) {
    if (predicate(array[l], l, array)) return l;
  }
  return -1;
}

export function buildResourceDir(entries: DirEntry[]): ResourceDir {
  const entriesByTypeAndNumber: Record<ResourceType, Map<number, DirEntry>> = {
    LOGIC: new Map<number, DirEntry>(),
    PIC: new Map<number, DirEntry>(),
    SOUND: new Map<number, DirEntry>(),
    VIEW: new Map<number, DirEntry>(),
  };
  entries.forEach((entry) => {
    entriesByTypeAndNumber[entry.resourceType].set(entry.resourceNumber, entry);
  });

  const dir: ResourceDir = {
    LOGIC: [],
    PIC: [],
    SOUND: [],
    VIEW: [],
  };

  [ResourceType.LOGIC, ResourceType.PIC, ResourceType.SOUND, ResourceType.VIEW].forEach(
    (resourceType) => {
      for (let resourceNumber = 0; resourceNumber < 256; resourceNumber++) {
        dir[resourceType].push(entriesByTypeAndNumber[resourceType].get(resourceNumber));
      }

      const lastResourceIndex = findLastIndex(dir[resourceType], (resource) => resource != null);
      dir[resourceType].splice(lastResourceIndex + 1);
    },
  );

  return dir;
}

export function writeV2Dir(entries: (DirEntry | undefined)[]): Buffer {
  const data = Buffer.alloc(3 * entries.length);

  entries.forEach((entry, index) => {
    const offset = index * 3;
    if (entry == null) {
      data.write('\xff\xff\xff', offset, 'binary');
    } else {
      data.writeUInt8((entry.volumeNumber << 4) + ((entry.offset & 0xf0000) >> 16), offset);
      data.writeUInt8((entry.offset & 0xff00) >> 8, offset + 1);
      data.writeUInt8(entry.offset & 0xff, offset + 2);
    }
  });

  return data;
}

export function writeV2DirFiles(
  outputPath: string,
  resourceDir: ResourceDir,
  logger: Logger,
): void {
  (
    [
      ['LOGDIR', resourceDir.LOGIC],
      ['PICDIR', resourceDir.PIC],
      ['SNDDIR', resourceDir.SOUND],
      ['VIEWDIR', resourceDir.VIEW],
    ] as const
  ).forEach(([fileName, entries]) => {
    const filePath = path.join(outputPath, fileName);
    const data = writeV2Dir(entries);
    logger.log(`Writing ${fileName} (${filesize(data.byteLength, { base: 2 })})`);
    fs.writeFileSync(filePath, data);
  });
}

export function writeV3DirFile(
  outputPath: string,
  gameId: string,
  resourceDir: ResourceDir,
  logger: Logger,
): void {
  const dirHeader = Buffer.alloc(8);
  const dirBlocks: Buffer[] = [];
  let offset = 8;

  ([resourceDir.LOGIC, resourceDir.PIC, resourceDir.VIEW, resourceDir.SOUND] as const).forEach(
    (entries, index) => {
      dirHeader.writeUInt16LE(offset, index * 2);
      const block = writeV2Dir(entries);
      offset += block.byteLength;
      dirBlocks.push(block);
    },
  );

  const data = Buffer.concat([dirHeader, ...dirBlocks]);
  logger.log(`Writing ${gameId}DIR (${filesize(data.byteLength, { base: 2 })})`);
  const filePath = path.join(outputPath, `${gameId}DIR`);
  fs.writeFileSync(filePath, data);
}

export function writeV2ResourceFiles(
  outputPath: string,
  resourceVolumes: (EncodedResource[] | undefined)[],
  logger: Logger,
): void {
  const resourceData: Buffer[] = [];
  const dirEntries: DirEntry[] = [];

  resourceVolumes.forEach((resources, volumeNumber) => {
    if (resources == null) {
      return;
    }
    const [volumeData, volumeDirEntries] = writeVolume(volumeNumber, resources);
    resourceData.push(volumeData);
    dirEntries.push(...volumeDirEntries);
  });

  const resourceDir = buildResourceDir(dirEntries);

  writeV2DirFiles(outputPath, resourceDir, logger);
  resourceData.forEach((data, volumeNumber) => {
    const filePath = path.join(outputPath, `VOL.${volumeNumber}`);
    logger.log(`Writing VOL.${volumeNumber} (${filesize(data.byteLength, { base: 2 })})`);
    fs.writeFileSync(filePath, data);
  });
}

export function writeV3ResourceFiles(
  outputPath: string,
  gameId: string,
  resourceVolumes: (EncodedResource[] | undefined)[],
  logger: Logger,
) {
  const resourceData: Buffer[] = [];
  const dirEntries: DirEntry[] = [];

  resourceVolumes.forEach((resources, volumeNumber) => {
    if (resources == null) {
      return;
    }
    const [volumeData, volumeDirEntries] = writeVolume(volumeNumber, resources);
    resourceData.push(volumeData);
    dirEntries.push(...volumeDirEntries);
  });

  const resourceDir = buildResourceDir(dirEntries);

  writeV3DirFile(outputPath, gameId, resourceDir, logger);
  resourceData.forEach((data, volumeNumber) => {
    const filePath = path.join(outputPath, `${gameId}VOL.${volumeNumber}`);
    logger.log(`Writing ${gameId}VOL.${volumeNumber} (${filesize(data.byteLength, { base: 2 })})`);
    fs.writeFileSync(filePath, data);
  });
}

export type ExplicitVolumeSpecification = {
  number: number;
  resources: { resourceType: ResourceType; resourceNumber: number }[];
};

// This implementation does not attempt to solve the knapsack problem, but in the future
// it might be smart to try.
export function encodeResourceVolumes(
  resources: Resource[],
  encode: (volumeNumber: number, resource: Resource) => EncodedResource,
  explicitVolumes: ExplicitVolumeSpecification[],
): (EncodedResource[] | undefined)[] {
  const volumes: (EncodedResource[] | undefined)[] = [];
  const resourceIndex = new Map(
    resources.map((resource) => [`${resource.type}-${resource.number}`, resource]),
  );
  const explicitResources = new Set<Resource>();

  explicitVolumes.forEach((explicitVolume) => {
    volumes[explicitVolume.number] = explicitVolume.resources.map((resourceSpecification) => {
      const resource = resourceIndex.get(
        `${resourceSpecification.resourceType}-${resourceSpecification.resourceNumber}`,
      );
      if (!resource) {
        throw new Error(
          `${resourceSpecification.resourceType} ${resourceSpecification.resourceNumber} does not exist`,
        );
      }

      explicitResources.add(resource);
      return encode(explicitVolume.number, resource);
    });
  });

  const findUnusedVolume = () => {
    const emptySpace = volumes.findIndex((volume) => volume == null);
    if (emptySpace === -1) {
      return volumes.length;
    }
    return emptySpace;
  };

  let currentVolume: EncodedResource[] = [];
  let currentVolumeNumber = findUnusedVolume();
  let currentVolumeSize = 0;

  for (const resource of resources) {
    if (explicitResources.has(resource)) {
      continue;
    }

    let encodedResource = encode(currentVolumeNumber, resource);
    if (encodedResource.encodedData.byteLength + currentVolumeSize > MAX_VOLUME_SIZE) {
      volumes[currentVolumeNumber] = currentVolume;
      currentVolume = [];
      currentVolumeSize = 0;
      currentVolumeNumber = findUnusedVolume();
      encodedResource = encode(currentVolumeNumber, resource);
    }
    currentVolume.push(encodedResource);
    currentVolumeSize += encodedResource.encodedData.byteLength;
  }

  if (currentVolume.length > 0) {
    volumes[currentVolumeNumber] = currentVolume;
  }

  return volumes;
}
