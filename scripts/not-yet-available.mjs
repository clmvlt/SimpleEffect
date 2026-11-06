// Placeholder for scripts whose tooling is set up by a later roadmap prompt.
// Usage: node scripts/not-yet-available.mjs <script-name> <prompt-number>
const [scriptName = 'this script', promptNumber = '??'] = process.argv.slice(2);

console.error(
  `"pnpm ${scriptName}" is not available yet: it is set up by roadmap prompt ${promptNumber} (see prompts/README.md).`,
);
process.exit(1);
