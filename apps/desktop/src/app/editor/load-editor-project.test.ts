import { CoreRequestError, type Command } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { session } from '../../shared/project-store/testing/session-fixture';
import { loadEditorProject } from './load-editor-project';

type Send = Parameters<typeof loadEditorProject>[0]['send'];

function client(answer: (command: Command) => unknown) {
  const sent: Command[] = [];
  // The fake answers whatever the test returns for each command.
  const send = (async (command: Command) => {
    sent.push(command);
    await Promise.resolve();
    return answer(command);
  }) as Send;
  return { sent, client: { send } };
}

const notOpen = new CoreRequestError('getProjectSnapshot', {
  code: 'PROJECT_NOT_OPEN',
  message: 'not open',
});

describe('loadEditorProject', () => {
  it('uses the session of a project the core has open', async () => {
    const fake = client(() => session());
    await expect(loadEditorProject(fake.client, 'project-1')).resolves.toEqual(session());
    expect(fake.sent).toEqual([{ kind: 'getProjectSnapshot', projectId: 'project-1' }]);
  });

  it('reopens a project from its recent folder when the core does not have it open', async () => {
    const fake = client((command) => {
      if (command.kind === 'getProjectSnapshot') throw notOpen;
      if (command.kind === 'listRecentProjects') {
        return { projects: [{ id: 'project-1', path: '/projects/Film.seproj' }] };
      }
      return session();
    });
    await loadEditorProject(fake.client, 'project-1');
    expect(fake.sent.at(-1)).toEqual({ kind: 'openProject', path: '/projects/Film.seproj' });
  });

  it('fails when the project is neither open nor recent', async () => {
    const fake = client((command) => {
      if (command.kind === 'getProjectSnapshot') throw notOpen;
      return { projects: [] };
    });
    await expect(loadEditorProject(fake.client, 'project-1')).rejects.toBe(notOpen);
  });
});
