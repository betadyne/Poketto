const REPO_URL = "https://github.com/betadyne/Poketto";

export const RELEASE_PAGE_URL = `${REPO_URL}/releases/latest`;

export function releaseTagUrl(version: string): string {
  const tag = version.startsWith("v") ? version : `v${version}`;
  return `${REPO_URL}/releases/tag/${tag}`;
}
