import { useContext, useEffect, useMemo, useRef } from 'react';
import { CommandListNavigationContext } from './CommandListNavigation';
import { describeCommand } from './describeCommand';
import { EditingPictureResource } from './EditingPictureTypes';
import { PicEditorControlContext } from './PicEditorControlContext';

export function PicCommandList({ pictureResource }: { pictureResource: EditingPictureResource }) {
  const { confirm, deleteCommand } = useContext(PicEditorControlContext);
  const {
    setAllCommandsEnabled,
    setCommandEnabled,
    jumpRelative,
    jumpTo,
    currentCommandId,
  } = useContext(CommandListNavigationContext);
  const commandElements = useRef(new Map<string, HTMLLIElement>());

  const firstCommand = useMemo(() => pictureResource.commands[0], [pictureResource.commands]);

  useEffect(() => {
    let commandElement: HTMLLIElement | undefined;
    if (currentCommandId) {
      commandElement = commandElements.current.get(currentCommandId);
    } else if (firstCommand) {
      commandElement = commandElements.current.get(firstCommand.uuid);
    }

    if (commandElement) {
      commandElement.scrollIntoView({ block: 'center' });
    }
  }, [currentCommandId, firstCommand]);

  return (
    <>
      <div style={{ display: 'flex' }}>
        <button
          type="button"
          style={{ margin: '1px' }}
          onClick={() => setAllCommandsEnabled(false)}
          aria-label="Go to start"
          title="Go to start"
          disabled={pictureResource.commands.length === 0 || currentCommandId == null}
        >
          <i className="bi-chevron-bar-up" role="img" />
        </button>
        <button
          type="button"
          style={{ margin: '1px' }}
          onClick={() => jumpRelative(-1)}
          disabled={pictureResource.commands.length === 0 || currentCommandId == null}
          title="Previous command"
          aria-label="Previous command"
        >
          <i className="bi-chevron-up" role="img" />
        </button>
        <button
          type="button"
          style={{ margin: '1px' }}
          onClick={() => jumpRelative(1)}
          disabled={
            pictureResource.commands.length === 0 ||
            currentCommandId === pictureResource.commands[pictureResource.commands.length - 1]?.uuid
          }
          title="Next command"
          aria-label="Next command"
        >
          <i className="bi-chevron-down" role="img" />
        </button>
        <button
          type="button"
          style={{ margin: '1px' }}
          onClick={() => setAllCommandsEnabled(true)}
          aria-label="Go to end"
          title="Go to end"
          disabled={
            pictureResource.commands.length === 0 ||
            currentCommandId === pictureResource.commands[pictureResource.commands.length - 1]?.uuid
          }
        >
          <i className="bi-chevron-bar-down" role="img" />
        </button>
      </div>
      <div style={{ overflowY: 'scroll' }}>
        <ul className="pic-editor-command-list">
          {pictureResource.commands.map((command, index) => (
            <li
              key={command.uuid}
              className={
                currentCommandId && command.uuid === currentCommandId
                  ? 'current'
                  : currentCommandId == null && index === 0
                  ? 'prev-current'
                  : ''
              }
              ref={(element) => {
                if (element) {
                  commandElements.current.set(command.uuid, element);
                } else {
                  commandElements.current.delete(command.uuid);
                }
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center' }}>
                <input
                  type="checkbox"
                  value={command.uuid}
                  checked={command.enabled}
                  onChange={(event) => setCommandEnabled(command.uuid, event.target.checked)}
                />
                <div style={{ flexGrow: 1 }}>{describeCommand(command)}</div>
                <button
                  type="button"
                  onClick={async () => {
                    if (await confirm('Are you sure you want to delete this command?')) {
                      deleteCommand(command.uuid);
                    }
                  }}
                  className="secondary pic-editor-command-action-button"
                >
                  <i className="bi-trash" role="img" aria-label="Delete command" />
                </button>
                <button
                  type="button"
                  onClick={() => jumpTo(command.uuid)}
                  className="secondary pic-editor-command-action-button"
                >
                  Go to
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>
    </>
  );
}
