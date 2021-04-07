import { PictureCommand, PictureResource } from 'agikit-core/dist/Types/Picture';
import assertNever from 'assert-never';
import { v4 as uuidv4 } from 'uuid';

export type EditingPictureCommand = PictureCommand & {
  uuid: string;
  enabled: boolean;
};

export type EditingPictureResource = Omit<PictureResource, 'commands'> & {
  commands: EditingPictureCommand[];
};

export type PicDocumentDeleteCommandEdit = {
  type: 'DeleteCommand';
  commandId: string;
};

export type PicDocumentAddCommandsEdit = {
  type: 'AddCommands';
  commands: EditingPictureCommand[];
  afterCommandId: string | undefined;
};

export type PicDocumentEdit = PicDocumentAddCommandsEdit | PicDocumentDeleteCommandEdit;

export function prepareCommandForEditing(
  command: PictureCommand,
): EditingPictureResource['commands'][number] {
  return {
    ...command,
    uuid: uuidv4(),
    enabled: true,
  };
}

export function applyEdit(
  resource: EditingPictureResource,
  edit: PicDocumentEdit,
): EditingPictureResource {
  if (edit.type === 'AddCommands') {
    let index = 0;
    if (edit.afterCommandId != null) {
      index = resource.commands.findIndex((c) => c.uuid === edit.afterCommandId);
      if (index === -1) {
        throw new Error(`Invalid edit: can't find command ${edit.afterCommandId} to insert after`);
      }
    }

    const newCommands = [...resource.commands];
    newCommands.splice(index + 1, 0, ...edit.commands);
    return { ...resource, commands: newCommands };
  }

  if (edit.type === 'DeleteCommand') {
    const index = resource.commands.findIndex((c) => c.uuid === edit.commandId);

    if (index === -1) {
      throw new Error(`Invalid edit: can't find command ${edit.commandId} to delete`);
    }

    const newCommands = [...resource.commands];
    newCommands.splice(index, 1);
    return { ...resource, commands: newCommands };
  }

  assertNever(edit);
}

export function applyEditsToResource(
  resource: EditingPictureResource,
  edits: PicDocumentEdit[],
): EditingPictureResource {
  return edits.reduce((workingResource, edit) => applyEdit(workingResource, edit), resource);
}
