import { Picture, PictureCommand, PictureCommandOpcodes } from '../..';

export type PictureJSONCommand = Omit<PictureCommand, 'opcode'>;
export type PictureJSON = {
  commands: PictureJSONCommand[];
};

export function readPictureJSON(json: PictureJSON): Picture {
  const commands: PictureCommand[] = [];

  for (const jsonCommand of json.commands) {
    const opcode = PictureCommandOpcodes[jsonCommand.type];
    if (opcode == null) {
      throw new Error(`Unknown picture command type: ${JSON.stringify(jsonCommand.type)}`);
    }

    commands.push({ ...jsonCommand, opcode } as PictureCommand);
  }

  return {
    commands,
  };
}

export function buildPictureJSON(picture: Picture): PictureJSON {
  return {
    commands: picture.commands.map(({ opcode, ...rest }) => rest),
  };
}
