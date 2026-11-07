import { describe, expect, it } from 'vitest';

import { buildContentSecurityPolicy } from './content-security-policy';

function directive(policy: string, name: string): string | undefined {
  return policy
    .split('; ')
    .find((entry) => entry.startsWith(`${name} `))
    ?.slice(name.length + 1);
}

describe('buildContentSecurityPolicy', () => {
  it('only lets a packaged app connect to the local core', () => {
    const policy = buildContentSecurityPolicy({ devServer: false });
    expect(directive(policy, 'connect-src')).toBe('ws://127.0.0.1:* http://127.0.0.1:*');
    expect(directive(policy, 'default-src')).toBe("'none'");
  });

  it('forbids inline scripts outside the dev server', () => {
    const policy = buildContentSecurityPolicy({ devServer: false });
    expect(directive(policy, 'script-src')).toBe("'self'");
    expect(policy).not.toContain('unsafe-eval');
    expect(policy).not.toContain('localhost');
  });

  it('allows the inline <style> elements inserted by Radix and sonner', () => {
    const policy = buildContentSecurityPolicy({ devServer: false });
    expect(directive(policy, 'style-src')).toBe("'self' 'unsafe-inline'");
  });

  it('loads fonts from the bundle and the font catalogue of the core only', () => {
    const policy = buildContentSecurityPolicy({ devServer: false });
    expect(directive(policy, 'font-src')).toBe("'self' http://127.0.0.1:*");
  });

  it('relaxes only what Vite Fast Refresh needs in development', () => {
    const policy = buildContentSecurityPolicy({ devServer: true });
    expect(directive(policy, 'script-src')).toBe("'self' 'unsafe-inline'");
    expect(directive(policy, 'connect-src')).toContain('ws://localhost:*');
    expect(policy).not.toContain('https:');
  });
});
