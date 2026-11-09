/** What a piece of template code is, for colouring. */
export type CodeTokenKind = 'keyword' | 'string' | 'number' | 'comment' | 'punctuation' | 'plain';

/** A piece of one line of code. */
export interface CodeToken {
  readonly kind: CodeTokenKind;
  readonly text: string;
}

const KEYWORDS = new Set([
  'as',
  'async',
  'await',
  'break',
  'case',
  'catch',
  'class',
  'const',
  'continue',
  'default',
  'do',
  'else',
  'export',
  'extends',
  'false',
  'for',
  'function',
  'if',
  'import',
  'in',
  'instanceof',
  'interface',
  'let',
  'new',
  'null',
  'of',
  'return',
  'switch',
  'this',
  'throw',
  'true',
  'try',
  'type',
  'typeof',
  'undefined',
  'var',
  'void',
  'while',
]);

/** Line comments, strings (one line), numbers, words, punctuation, spaces. */
const PATTERN =
  /(\/\/.*$)|('(?:[^'\\]|\\.)*'?|"(?:[^"\\]|\\.)*"?|`(?:[^`\\]|\\.)*`?)|(\b\d[\d_]*(?:\.\d+)?(?:e[+-]?\d+)?\b)|([A-Za-z_$][\w$]*)|([{}()[\];,.:?=<>+\-*/%!&|^~]+)|(\s+)|(.)/gy;

/**
 * The tokens of each line of `source` for the read-only code view (prompt 24): keywords,
 * strings, numbers, comments (`//` and `/* … *\/` across lines), punctuation, the rest plain. A
 * light tokenizer — not a parser: template literals spanning lines are coloured line by line.
 */
export function codeLines(source: string): CodeToken[][] {
  let inBlockComment = false;
  return source.split('\n').map((line) => {
    const tokens: CodeToken[] = [];
    let rest = line;
    if (inBlockComment) {
      const end = rest.indexOf('*/');
      if (end === -1) return [{ kind: 'comment', text: rest }];
      tokens.push({ kind: 'comment', text: rest.slice(0, end + 2) });
      rest = rest.slice(end + 2);
      inBlockComment = false;
    }
    while (rest !== '') {
      const start = rest.indexOf('/*');
      const head = start === -1 ? rest : rest.slice(0, start);
      tokens.push(...lineTokens(head));
      if (start === -1) break;
      const end = rest.indexOf('*/', start + 2);
      if (end === -1) {
        tokens.push({ kind: 'comment', text: rest.slice(start) });
        inBlockComment = true;
        break;
      }
      tokens.push({ kind: 'comment', text: rest.slice(start, end + 2) });
      rest = rest.slice(end + 2);
    }
    return tokens;
  });
}

function lineTokens(text: string): CodeToken[] {
  const tokens: CodeToken[] = [];
  PATTERN.lastIndex = 0;
  for (let match = PATTERN.exec(text); match !== null; match = PATTERN.exec(text)) {
    const [whole, comment, string, number, word, punctuation] = match;
    const kind: CodeTokenKind =
      comment !== undefined
        ? 'comment'
        : string !== undefined
          ? 'string'
          : number !== undefined
            ? 'number'
            : word !== undefined && KEYWORDS.has(word)
              ? 'keyword'
              : punctuation !== undefined
                ? 'punctuation'
                : 'plain';
    const previous = tokens.at(-1);
    if (previous?.kind === kind && kind === 'plain') {
      tokens[tokens.length - 1] = { kind, text: previous.text + whole };
    } else tokens.push({ kind, text: whole });
  }
  return tokens;
}
