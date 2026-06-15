/** v2free fork branding & external links (keep in one place for upstream merges). */
export const FORK_GITHUB_REPO = 'https://github.com/fqfqgo/clash-verge-rev'
export const FORK_GITHUB_RELEASES = `${FORK_GITHUB_REPO}/releases`
export const FORK_DOC_URL = 'https://cdn.v2ai.top/doc/#/clash-verge'
export const FORK_PRODUCT_SUFFIX = 'for v2free'

export function forkVersionLabel(version: string) {
  return `${FORK_PRODUCT_SUFFIX} v${version}`
}

export function forkWindowTitle(version: string) {
  return `Clash Verge ${forkVersionLabel(version)}`
}

export function forkReleaseTagUrl(version: string) {
  return `${FORK_GITHUB_REPO}/releases/tag/v${version}`
}
