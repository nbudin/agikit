import {
  PictureCommand,
  PictureResource,
} from "agikit-core/dist/Types/Picture";

type EditingPictureCommand = PictureCommand & {
  uuid: string;
  enabled: boolean;
};

export type EditingPictureResource = Omit<PictureResource, "commands"> & {
  commands: EditingPictureCommand[];
};
