/** `text` lowercased without accents, so « eté » finds « Été » (accent- and case-insensitive search). */
export function searchable(text: string): string {
  return text
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}
