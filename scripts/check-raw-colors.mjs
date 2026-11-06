// Fails when UI source code contains a raw colour literal outside a `tokens/` folder (run by
// `pnpm check`). Usage: node scripts/check-raw-colors.mjs [root] — `root` defaults to the repository.
import { resolve } from 'node:path';

import { scanForRawColors, SCANNED_DIRECTORIES } from './lib/raw-colors.mjs';
import { repoRoot } from './lib/run-command.mjs';

const root = resolve(process.argv[2] ?? repoRoot);
const findings = await scanForRawColors(root);

if (findings.length > 0) {
  for (const { file, line, column, literal } of findings) {
    console.error(`${file}:${line}:${column}  raw colour "${literal}"`);
  }
  console.error(
    `\n✖ ${findings.length} raw colour literal(s) in ${SCANNED_DIRECTORIES.join(', ')}.\n` +
      '  Use a design token instead (bg-panel, text-text-muted, var(--color-accent)…).\n' +
      '  New colours are added to packages/ui/src/tokens/colors.css (see packages/ui/CLAUDE.md).',
  );
  process.exit(1);
}
console.log(`No raw colour literal in ${SCANNED_DIRECTORIES.join(', ')}.`);
