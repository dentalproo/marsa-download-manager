/** The archives Marsa reads: ZIP, TAR and `.tar.gz` (and `.tgz`, the same
 * thing under its short name). RAR is not read — the unRAR licence forbids
 * works derived from it — and 7z is not built, so neither shows the button.
 */
export function isArchiveName(name: string): boolean {
  return /\.(zip|tar|tar\.gz|tgz)$/i.test(name);
}
