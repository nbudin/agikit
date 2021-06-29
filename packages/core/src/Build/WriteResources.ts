import fs from 'fs';
import path from 'path';
import { DirEntry, Resource, ResourceDir, ResourceType } from '../Types/Resources';

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

export function writeV2Volume(
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
    // console.log(
    //   `${encodedResource.type} ${
    //     encodedResource.number
    //   }: VOL.${volumeNumber} offset ${offset.toString(16)}`,
    // );
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

export function writeV2DirFiles(outputPath: string, resourceDir: ResourceDir): void {
  ([
    ['LOGDIR', resourceDir.LOGIC],
    ['PICDIR', resourceDir.PIC],
    ['SNDDIR', resourceDir.SOUND],
    ['VIEWDIR', resourceDir.VIEW],
  ] as const).forEach(([fileName, entries]) => {
    const filePath = path.join(outputPath, fileName);
    const data = writeV2Dir(entries);
    fs.writeFileSync(filePath, data);
  });
}

export function writeV2ResourceFiles(
  outputPath: string,
  resourceVolumes: (EncodedResource[] | undefined)[],
): void {
  const resourceData: Buffer[] = [];
  const dirEntries: DirEntry[] = [];

  resourceVolumes.forEach((resources, volumeNumber) => {
    if (resources == null) {
      return;
    }
    const [volumeData, volumeDirEntries] = writeV2Volume(volumeNumber, resources);
    resourceData.push(volumeData);
    dirEntries.push(...volumeDirEntries);
  });

  const resourceDir = buildResourceDir(dirEntries);

  writeV2DirFiles(outputPath, resourceDir);
  resourceData.forEach((data, volumeNumber) => {
    const filePath = path.join(outputPath, `VOL.${volumeNumber}`);
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
