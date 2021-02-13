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

type ResourceCommon = {
  data: Buffer;
  number: number;
};

export type LogicResource = ResourceCommon & {
  type: ResourceType.LOGIC;
};

export type PicResource = ResourceCommon & {
  type: ResourceType.PIC;
};

export type ViewResource = ResourceCommon & {
  type: ResourceType.VIEW;
};

export type SoundResource = ResourceCommon & {
  type: ResourceType.SOUND;
};

export type Resource = LogicResource | PicResource | ViewResource | SoundResource;
