// Pure helpers of `scripts/release-notes.mjs` (prompt 30): the GitHub release text, from the
// Conventional Commits since the previous tag, with the installation notes of every platform.

/** Commit types shown in the notes, in order, with their (French, user-facing) headings. */
const SECTIONS = [
  ['feat', 'Nouveautés'],
  ['fix', 'Corrections'],
  ['perf', 'Performances'],
];

const CONVENTIONAL = /^(?<type>[a-z]+)(?:\((?<scope>[^)]+)\))?(?<breaking>!)?: (?<subject>.+)$/;

/**
 * @typedef {object} ReleaseCommit
 * @property {string} subject First line of the message.
 * @property {string} hash Short hash.
 */

/**
 * Commits grouped by section; types other than feat/fix/perf (docs, chore, test, ci…) and
 * messages that are not Conventional Commits are left out: they do not change the app.
 *
 * @param {readonly ReleaseCommit[]} commits newest first, as `git log` prints them
 * @returns {Array<{ title: string, lines: string[] }>}
 */
export function groupCommits(commits) {
  return SECTIONS.flatMap(([type, title]) => {
    const lines = commits.flatMap((commit) => {
      const match = CONVENTIONAL.exec(commit.subject);
      if (match?.groups?.type !== type) return [];
      const { scope, breaking, subject } = match.groups;
      const prefix = scope === undefined ? '' : `**${scope}** : `;
      const mark = breaking === undefined ? '' : ' ⚠️';
      return [`- ${prefix}${subject}${mark} (${commit.hash})`];
    });
    return lines.length === 0 ? [] : [{ title, lines }];
  });
}

/**
 * The installer names of a version (`electron-builder.yml` `artifactName`s).
 *
 * @param {string} version without the `v`
 */
export function installerNames(version) {
  return {
    windows: `SimpleEffect-${version}-windows-x64-setup.exe`,
    macArm: `SimpleEffect-${version}-macos-arm64.dmg`,
    macIntel: `SimpleEffect-${version}-macos-x64.dmg`,
    appImage: `SimpleEffect-${version}-linux-x86_64.AppImage`,
    deb: `SimpleEffect-${version}-linux-amd64.deb`,
  };
}

/**
 * The release notes in Markdown.
 *
 * @param {{ tag: string, previousTag: string | null, commits: readonly ReleaseCommit[] }} input
 * @returns {string}
 */
export function releaseNotes({ tag, previousTag, commits }) {
  const version = tag.replace(/^v/, '');
  const names = installerNames(version);
  const sections = groupCommits(commits);
  const changes =
    sections.length === 0
      ? ['_Pas de changement visible dans l’application._']
      : sections.flatMap((section) => [`### ${section.title}`, '', ...section.lines, '']);
  const since = previousTag === null ? 'Première version.' : `Depuis ${previousTag}.`;
  return [
    `## SimpleEffect ${version}`,
    '',
    since,
    '',
    ...changes,
    '## Installation',
    '',
    `- **Windows 10/11 (x64)** : \`${names.windows}\`. Installation pour votre compte, sans droits administrateur. L’installeur n’est pas signé : si Windows SmartScreen s’affiche, cliquez sur « Informations complémentaires » puis « Exécuter quand même ».`,
    `- **macOS (Apple Silicon)** : \`${names.macArm}\` — **macOS (Intel)** : \`${names.macIntel}\` (sans recadrage automatique : ONNX Runtime n’existe pas pour les Mac Intel). Glissez SimpleEffect dans Applications. L’app n’est pas notariée : au premier lancement, faites un clic droit sur SimpleEffect → « Ouvrir », ou lancez dans le Terminal :`,
    '',
    '  ```bash',
    '  xattr -dr com.apple.quarantine /Applications/SimpleEffect.app',
    '  ```',
    '',
    `- **Linux (x86_64)** : \`${names.appImage}\` (\`chmod +x\` puis lancez-le) ou \`${names.deb}\` (\`sudo apt install ./${names.deb}\`).`,
    '',
    'L’IA passe par Claude Code, installé et connecté sur votre ordinateur ; l’assistant de bienvenue vérifie tout au premier lancement. Les modèles de transcription se téléchargent depuis l’app, avec votre accord.',
    '',
  ].join('\n');
}
