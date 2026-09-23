/** Join Astro's deployment base with a site-absolute path. */
export function withBasePath(base: string, path: string): string {
  const normalizedBase = base === "/" ? "" : base.replace(/\/$/, "");
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;
  return `${normalizedBase}${normalizedPath}`;
}
