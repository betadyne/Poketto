export function isVndbId(query: string): boolean {
  return /^v\d+$/i.test(query.trim());
}
