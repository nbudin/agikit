export type ObjectListEntry = {
  name: string;
  startingRoomNumber: number;
};

export type ObjectList = {
  objects: ObjectListEntry[];
  maxAnimatedObjects: number;
};
