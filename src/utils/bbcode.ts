const BBCODE_REGEX = /\[(url|spoiler|quote|raw|code)(?:=[^\]]*)?]|\[\/(url|spoiler|quote|raw|code)]/gi;

export function stripBBCode(text: string): string {
  return text.replace(BBCODE_REGEX, "");
}
