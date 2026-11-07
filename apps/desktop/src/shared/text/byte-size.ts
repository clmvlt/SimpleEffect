/** Units of {@link byteSize}, binary multiples, French abbreviations (octets). */
const UNITS = ['o', 'Ko', 'Mo', 'Go', 'To'] as const;

/** Bytes as a short size in `locale`: `512 o`, `12,4 Mo`, `3,1 Go` (binary multiples). */
export function byteSize(bytes: number, locale: string): string {
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < UNITS.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const number = new Intl.NumberFormat(locale, {
    maximumFractionDigits: unit === 0 ? 0 : 1,
  }).format(value);
  return `${number} ${UNITS[unit] ?? 'o'}`;
}
