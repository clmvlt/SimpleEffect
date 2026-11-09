import { transform } from 'sucrase';

/** A compiled template module: CommonJS JavaScript with the same lines as the TypeScript. */
export interface CompiledTemplate {
  readonly code: string;
}

/** A problem found while compiling, at a line of the module (1-based). */
export interface CompileProblem {
  readonly message: string;
  readonly line: number | null;
  readonly column: number | null;
}

/** The outcome of {@link compileTemplate}. */
export type CompileResult =
  | { readonly ok: true; readonly compiled: CompiledTemplate }
  | { readonly ok: false; readonly problems: readonly CompileProblem[] };

/** `import x from 'y'`, `import 'y'`, `import(…)`, `export … from 'y'` at a line's start. */
const IMPORTS = /^\s*(?:import(?:\s+[\w{*'"]|\s*\(|\s*['"])|export\s+(?:\*|\{[^}]*\})\s*from\b)/;
/** `require(` outside a property access. */
const REQUIRE = /(?:^|[^.\w$])require\s*\(/;

/**
 * Compiles a template's TypeScript to JavaScript with the embedded `sucrase` (no type checking,
 * no network): types removed, ES module exports turned into `exports.name = …`. Lines keep their
 * numbers, so runtime errors point at the agent's own lines. Modules may not import anything:
 * everything a template uses comes through `ctx`.
 */
export function compileTemplate(source: string): CompileResult {
  const imports = importProblems(source);
  if (imports.length > 0) return { ok: false, problems: imports };
  try {
    const { code } = transform(source, {
      transforms: ['typescript', 'imports'],
      production: true,
      disableESTransforms: true,
    });
    return { ok: true, compiled: { code } };
  } catch (error) {
    return { ok: false, problems: [syntaxProblem(error)] };
  }
}

function importProblems(source: string): CompileProblem[] {
  const problems: CompileProblem[] = [];
  source.split('\n').forEach((text, index) => {
    if (IMPORTS.test(text) || REQUIRE.test(text)) {
      problems.push({
        message:
          'templates cannot import or require modules: everything they use comes through `ctx` ' +
          '(drawing, text layout, easing, lerp, clamp, random)',
        line: index + 1,
        column: null,
      });
    }
  });
  return problems;
}

/** Sucrase's syntax error, without the `(line:column)` it appends to its message. */
function syntaxProblem(error: unknown): CompileProblem {
  const message = error instanceof Error ? error.message : String(error);
  const location = locationOf(error);
  return {
    message: `syntax error: ${message.replace(/\s*\(\d+:\d+\)\s*$/, '')}`,
    line: location?.line ?? null,
    column: location === null ? null : location.column + 1,
  };
}

function locationOf(error: unknown): { line: number; column: number } | null {
  if (typeof error !== 'object' || error === null || !('loc' in error)) return null;
  const loc: unknown = error.loc;
  if (typeof loc !== 'object' || loc === null) return null;
  const line = 'line' in loc && typeof loc.line === 'number' ? loc.line : null;
  const column = 'column' in loc && typeof loc.column === 'number' ? loc.column : 0;
  return line === null ? null : { line, column };
}
