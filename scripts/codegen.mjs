// `pnpm codegen [--check]`: exports the Rust protocol and domain types to
// packages/protocol/src/generated/ (ADR-0010).
// TODO(prompt-02): run the ts-rs export, and with --check fail when the generated files are stale.
const check = process.argv.includes('--check');

console.log(
  check
    ? 'codegen --check: nothing to verify yet (the ts-rs export is added by prompt 02).'
    : 'codegen: nothing to generate yet (the ts-rs export is added by prompt 02).',
);
