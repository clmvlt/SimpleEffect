/** File name of an executable on `platform`: Windows needs the `.exe` extension. */
export function executableName(baseName: string, platform: NodeJS.Platform): string {
  return platform === 'win32' ? `${baseName}.exe` : baseName;
}
