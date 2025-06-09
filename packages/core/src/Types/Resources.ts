import { ResourceType, DirEntry } from 'agikit_core';

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
