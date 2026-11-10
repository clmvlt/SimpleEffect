import { resolve } from 'node:path';

import { describe, expect, it, vi } from 'vitest';

import type { ExportActivity } from '../shared/bridge-api';
import { ExportActivityGuard, type ExportGuardServices } from './export-activity';
import { parseExportActivity, parseResolveFilePath } from './export-arguments';

const ACTIVITY: ExportActivity = {
  title: 'Un export est en cours',
  message: 'Quitter annule l’export.',
  quitLabel: 'Quitter',
  stayLabel: 'Rester',
};

function services(answer: boolean) {
  return {
    preventSuspension: vi.fn(() => 7),
    allowSuspension: vi.fn(),
    confirmLeave: vi.fn(() => answer),
  } satisfies ExportGuardServices;
}

describe('parseResolveFilePath', () => {
  const folder = resolve('exports');

  it('accepts an absolute folder and a plain file name', () => {
    expect(parseResolveFilePath({ folder, fileName: 'Montage.mp4' })).toEqual({
      folder,
      fileName: 'Montage.mp4',
    });
  });

  it('refuses relative folders and names holding folders', () => {
    expect(() => parseResolveFilePath({ folder: 'exports', fileName: 'a.mp4' })).toThrow(
      /absolute/,
    );
    for (const fileName of ['../a.mp4', 'sub/a.mp4', 'sub\\a.mp4', '..', '', 'x'.repeat(256), 3]) {
      expect(() => parseResolveFilePath({ folder, fileName })).toThrow(/file name/);
    }
    expect(() => parseResolveFilePath(null)).toThrow(/options/);
  });
});

describe('parseExportActivity', () => {
  it('accepts null and four short texts', () => {
    expect(parseExportActivity(null)).toBeNull();
    expect(parseExportActivity(ACTIVITY)).toEqual(ACTIVITY);
  });

  it('refuses anything else', () => {
    expect(() => parseExportActivity('busy')).toThrow(/activity/);
    expect(() => parseExportActivity({ ...ACTIVITY, title: 1 })).toThrow(/string/);
    expect(() => parseExportActivity({ ...ACTIVITY, message: 'x'.repeat(501) })).toThrow(/500/);
  });
});

describe('ExportActivityGuard', () => {
  it('keeps the computer awake while exports run', () => {
    const fake = services(true);
    const guard = new ExportActivityGuard(fake);
    guard.set(ACTIVITY);
    guard.set({ ...ACTIVITY, title: '2 exports sont en cours' });
    expect(fake.preventSuspension).toHaveBeenCalledTimes(1);
    guard.set(null);
    expect(fake.allowSuspension).toHaveBeenCalledWith(7);
    guard.set(null);
    expect(fake.allowSuspension).toHaveBeenCalledTimes(1);
  });

  it('leaves at once without exports', () => {
    const fake = services(false);
    expect(new ExportActivityGuard(fake).mayLeave(null)).toBe(true);
    expect(fake.confirmLeave).not.toHaveBeenCalled();
  });

  it('asks while exports run and stays when told to', () => {
    const fake = services(false);
    const guard = new ExportActivityGuard(fake);
    guard.set(ACTIVITY);
    expect(guard.mayLeave(null)).toBe(false);
    expect(guard.mayLeave(null)).toBe(false);
    expect(fake.confirmLeave).toHaveBeenCalledTimes(2);
    expect(fake.confirmLeave).toHaveBeenLastCalledWith(null, ACTIVITY);
  });

  it('asks once per leave: the quit after a confirmed close goes through', () => {
    const fake = services(true);
    const guard = new ExportActivityGuard(fake);
    guard.set(ACTIVITY);
    expect(guard.mayLeave(null)).toBe(true);
    expect(guard.mayLeave(null)).toBe(true);
    expect(fake.confirmLeave).toHaveBeenCalledTimes(1);
  });
});
