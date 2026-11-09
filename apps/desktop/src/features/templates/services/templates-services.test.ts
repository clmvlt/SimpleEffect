import type { TemplateCheckOutcome, TemplateCheckRequest } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import { codeLines } from './code-tokens';
import { answerTemplateCheck, type TemplateCheckDeps } from './template-checks';

describe('codeLines', () => {
  it('colours keywords, strings, numbers, comments and punctuation line by line', () => {
    const [first, second] = codeLines("export const a = 'hi'; // note\nlet n = 4.5;");
    expect(first).toEqual([
      { kind: 'keyword', text: 'export' },
      { kind: 'plain', text: ' ' },
      { kind: 'keyword', text: 'const' },
      { kind: 'plain', text: ' a ' },
      { kind: 'punctuation', text: '=' },
      { kind: 'plain', text: ' ' },
      { kind: 'string', text: "'hi'" },
      { kind: 'punctuation', text: ';' },
      { kind: 'plain', text: ' ' },
      { kind: 'comment', text: '// note' },
    ]);
    expect(second?.find((token) => token.kind === 'number')?.text).toBe('4.5');
  });

  it('keeps block comments across lines', () => {
    const lines = codeLines('a /* one\ntwo */ b');
    expect(lines[0]?.at(-1)).toEqual({ kind: 'comment', text: '/* one' });
    expect(lines[1]?.[0]).toEqual({ kind: 'comment', text: 'two */' });
    expect(lines[1]?.at(-1)).toEqual({ kind: 'plain', text: ' b' });
  });
});

const REQUEST: TemplateCheckRequest = {
  requestId: 'check-1',
  projectId: 'project-1',
  source: 'export function render() {}',
  width: 1920,
  height: 1080,
  pictureWidth: 640,
  pictureHeight: 360,
};

function deps(check: TemplateCheckDeps['check']) {
  const report = vi.fn<TemplateCheckDeps['report']>(() => Promise.resolve());
  const resolved: TemplateCheckDeps = {
    endpoint: () => ({ url: 'http://127.0.0.1:1', token: 'secret' }),
    families: () => [],
    check,
    report,
  };
  return { resolved, report };
}

describe('answerTemplateCheck', () => {
  it('reports the verdict of the check', async () => {
    const passed: TemplateCheckOutcome = {
      kind: 'passed',
      metaJson: '{}',
      paramsJson: '[]',
      frames: [],
      slowestFrameMs: 1,
    };
    const check = vi.fn<TemplateCheckDeps['check']>(() => Promise.resolve(passed));
    const { resolved, report } = deps(check);
    await answerTemplateCheck(REQUEST, resolved);
    expect(check).toHaveBeenCalledWith(REQUEST, [], { url: 'http://127.0.0.1:1', token: 'secret' });
    expect(report).toHaveBeenCalledWith('check-1', passed);
  });

  it('reports a check that cannot run as a failure the agent reads', async () => {
    const { resolved, report } = deps(() => Promise.reject(new Error('no Workers here')));
    await answerTemplateCheck(REQUEST, resolved);
    expect(report).toHaveBeenCalledWith('check-1', {
      kind: 'failed',
      problems: [
        {
          stage: 'render',
          message: 'the editor could not run the template: no Workers here',
          line: null,
          column: null,
          time: null,
        },
      ],
    });
  });
});
