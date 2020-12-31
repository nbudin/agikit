export enum ResourceType {
  LOGIC = 'LOGIC',
  PIC = 'PIC',
  VIEW = 'VIEW',
  SOUND = 'SOUND',
}

export type DirEntry = {
  resourceType: ResourceType;
  resourceNumber: number;
  volumeNumber: number;
  offset: number;
};

export type ResourceDir = Record<ResourceType, (DirEntry | undefined)[]>;
