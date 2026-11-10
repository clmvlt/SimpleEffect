// Pure helpers of `scripts/generate-notices.mjs` (prompt 30): which Cargo crates ship, which
// files hold a package's licence, and the notices documents themselves.

/**
 * @typedef {object} NoticeEntry
 * @property {'component' | 'npm' | 'cargo'} source Where the entry comes from.
 * @property {string} name
 * @property {string} version
 * @property {string} license SPDX expression or free text.
 * @property {string} [url] Home page or repository.
 * @property {string} [note]
 * @property {Array<{ file: string, text: string }>} texts Licence and notice files.
 */

/**
 * @typedef {object} NoticesDocument
 * @property {number} formatVersion
 * @property {Array<Omit<NoticeEntry, 'texts'> & { textIds: number[] }>} entries
 * @property {Record<string, string>} texts Licence texts by id, shared between entries.
 */

/** Current `formatVersion` of `third-party-notices.json` (read by the desktop main process). */
export const NOTICES_FORMAT_VERSION = 1;

const LICENCE_FILE = /^(licen[cs]e|copying|notice|copyright|unlicense)([-_.].*)?$/i;

/**
 * The licence and notice files among `fileNames` (`LICENSE`, `LICENSE-MIT`, `COPYING.txt`,
 * `NOTICE`…), sorted.
 *
 * @param {readonly string[]} fileNames
 * @returns {string[]}
 */
export function licenceFilesIn(fileNames) {
  return fileNames.filter((name) => LICENCE_FILE.test(name)).sort();
}

/**
 * The licence of an npm `package.json`: `license`, or the old `license: { type }` and
 * `licenses: [{ type }]` forms.
 *
 * @param {Record<string, unknown>} manifest
 * @returns {string}
 */
export function npmLicenceOf(manifest) {
  const { license, licenses } = manifest;
  if (typeof license === 'string') return license;
  if (isRecord(license) && typeof license.type === 'string') return license.type;
  if (Array.isArray(licenses)) {
    const types = licenses.flatMap((item) =>
      isRecord(item) && typeof item.type === 'string' ? [item.type] : [],
    );
    if (types.length > 0) return types.join(' OR ');
  }
  return 'UNKNOWN';
}

/**
 * The home page of an npm `package.json` (`homepage`, else its repository as a web URL).
 *
 * @param {Record<string, unknown>} manifest
 * @returns {string | undefined}
 */
export function npmUrlOf(manifest) {
  if (typeof manifest.homepage === 'string') return manifest.homepage;
  const repository = isRecord(manifest.repository) ? manifest.repository.url : manifest.repository;
  if (typeof repository !== 'string') return undefined;
  return repository.replace(/^git\+/, '').replace(/\.git$/, '');
}

/**
 * The crates linked into `rootName`'s binary, from `cargo metadata --format-version 1
 * --filter-platform <host>`: its normal dependencies, transitively. Build and dev
 * dependencies, procedural macros (and what only they use) and workspace members are left
 * out: they do not ship.
 *
 * @param {CargoMetadata} metadata
 * @param {string} rootName
 * @returns {CargoPackage[]} sorted by name, then version
 */
export function shippedCrates(metadata, rootName) {
  const packages = new Map(metadata.packages.map((item) => [item.id, item]));
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const members = new Set(metadata.workspace_members);
  const root = metadata.packages.find((item) => item.name === rootName && members.has(item.id));
  if (root === undefined) throw new Error(`cargo metadata lists no workspace crate ${rootName}`);
  const seen = new Set([root.id]);
  const pending = [root.id];
  while (pending.length > 0) {
    const node = nodes.get(/** @type {string} */ (pending.pop()));
    for (const dependency of node?.deps ?? []) {
      const linked = dependency.dep_kinds.some((kind) => kind.kind === null);
      const target = packages.get(dependency.pkg);
      if (!linked || target === undefined || seen.has(target.id) || isProcMacro(target)) continue;
      seen.add(target.id);
      pending.push(target.id);
    }
  }
  return [...seen]
    .filter((id) => !members.has(id))
    .flatMap((id) => {
      const found = packages.get(id);
      return found === undefined ? [] : [found];
    })
    .sort(
      (left, right) =>
        compareText(left.name, right.name) || compareText(left.version, right.version),
    );
}

/**
 * The notices document: entries sorted (components first, then npm, then Cargo), identical
 * licence texts stored once.
 *
 * @param {readonly NoticeEntry[]} entries
 * @returns {NoticesDocument}
 */
export function noticesDocument(entries) {
  /** @type {Map<string, number>} */
  const idsByText = new Map();
  /** @type {Record<string, string>} */
  const texts = {};
  const sorted = [...entries].sort(compareEntries);
  const documentEntries = sorted.map(({ texts: files, ...entry }) => {
    const textIds = files.map((file) => {
      const text = normaliseText(file.text);
      let id = idsByText.get(text);
      if (id === undefined) {
        id = idsByText.size + 1;
        idsByText.set(text, id);
        texts[String(id)] = text;
      }
      return id;
    });
    return { ...entry, textIds: [...new Set(textIds)] };
  });
  return { formatVersion: NOTICES_FORMAT_VERSION, entries: documentEntries, texts };
}

/**
 * `THIRD-PARTY-NOTICES.txt`: every entry with its licence, then each licence text once.
 *
 * @param {NoticesDocument} document
 * @returns {string}
 */
export function noticesText(document) {
  const lines = [
    'SimpleEffect — third-party notices',
    '',
    'SimpleEffect ships or downloads the components and packages below. Each lists its licence',
    'and the numbers of its licence texts, printed after the list.',
    '',
  ];
  for (const entry of document.entries) {
    const version = entry.version === '' ? '' : ` ${entry.version}`;
    lines.push(`- [${entry.source}] ${entry.name}${version} — ${entry.license}`);
    if (entry.url !== undefined) lines.push(`  ${entry.url}`);
    if (entry.note !== undefined) lines.push(`  ${entry.note}`);
    if (entry.textIds.length > 0)
      lines.push(`  Texts: ${entry.textIds.map((id) => `#${id}`).join(', ')}`);
  }
  for (const [id, text] of Object.entries(document.texts)) {
    lines.push('', `${'='.repeat(30)} Text #${id} ${'='.repeat(30)}`, '', text);
  }
  return `${lines.join('\n')}\n`;
}

/** @param {string} text */
function normaliseText(text) {
  return text
    .replace(/^\u{FEFF}/u, '')
    .replaceAll('\r\n', '\n')
    .trim();
}

const SOURCE_ORDER = { component: 0, npm: 1, cargo: 2 };

/** @param {NoticeEntry} left @param {NoticeEntry} right */
function compareEntries(left, right) {
  return (
    SOURCE_ORDER[left.source] - SOURCE_ORDER[right.source] ||
    (left.source === 'component' ? 0 : compareText(left.name, right.name)) ||
    compareText(left.version, right.version)
  );
}

/** @param {string} left @param {string} right */
function compareText(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

/** @param {CargoPackage} item */
function isProcMacro(item) {
  return item.targets.some((target) => target.kind.includes('proc-macro'));
}

/** @param {unknown} value @returns {value is Record<string, unknown>} */
function isRecord(value) {
  return typeof value === 'object' && value !== null;
}

/**
 * @typedef {object} CargoPackage
 * @property {string} id
 * @property {string} name
 * @property {string} version
 * @property {string | null} license
 * @property {string | null} [license_file]
 * @property {string | null} [repository]
 * @property {string | null} [homepage]
 * @property {string} manifest_path
 * @property {Array<{ kind: string[] }>} targets
 */

/**
 * @typedef {object} CargoMetadata
 * @property {CargoPackage[]} packages
 * @property {string[]} workspace_members
 * @property {{ nodes: Array<{ id: string, deps: Array<{ pkg: string, dep_kinds: Array<{ kind: string | null }> }> }> }} resolve
 */
