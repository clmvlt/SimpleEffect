import { describe, expect, it } from 'vitest';

import { groupCommits, installerNames, releaseNotes } from './release-notes.mjs';

const commits = [
  { subject: 'docs(prompts): prompt 30 done', hash: 'aaa1111' },
  { subject: 'feat(about): licences and diagnostics', hash: 'bbb2222' },
  { subject: 'fix(core): keep the journal on disk-full!', hash: 'ccc3333' },
  { subject: 'perf!: proxies by default', hash: 'ddd4444' },
  { subject: 'Merge branch dev', hash: 'eee5555' },
  { subject: 'feat: first-run wizard', hash: 'fff6666' },
];

describe('release notes', () => {
  it('groups user-visible Conventional Commits', () => {
    expect(groupCommits(commits)).toEqual([
      {
        title: 'Nouveautés',
        lines: ['- **about** : licences and diagnostics (bbb2222)', '- first-run wizard (fff6666)'],
      },
      { title: 'Corrections', lines: ['- **core** : keep the journal on disk-full! (ccc3333)'] },
      { title: 'Performances', lines: ['- proxies by default ⚠️ (ddd4444)'] },
    ]);
  });

  it('names the installers and explains each platform', () => {
    expect(installerNames('1.2.0-rc.1').macIntel).toBe('SimpleEffect-1.2.0-rc.1-macos-x64.dmg');
    const notes = releaseNotes({ tag: 'v1.2.0', previousTag: 'v1.1.0', commits });
    expect(notes).toContain('## SimpleEffect 1.2.0');
    expect(notes).toContain('Depuis v1.1.0.');
    expect(notes).toContain('`SimpleEffect-1.2.0-windows-x64-setup.exe`');
    expect(notes).toContain('xattr -dr com.apple.quarantine /Applications/SimpleEffect.app');
    expect(notes).toContain('`SimpleEffect-1.2.0-linux-amd64.deb`');
    expect(releaseNotes({ tag: 'v0.1.0', previousTag: null, commits: [] })).toContain(
      'Pas de changement visible',
    );
  });
});
