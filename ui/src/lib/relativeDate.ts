/** Format an ISO-8601 string or unix-seconds number as a terse "N<unit> ago".
 * Empty/invalid input returns "—" so list rows don't render the literal
 * string "Invalid Date". Sub-minute intervals collapse to "just now" so the
 * UI doesn't churn on every render. */
export function relativeDate(input: string | number | undefined | null): string {
  if (input === null || input === undefined || input === "" || input === 0) return "—";
  const t = typeof input === "number" ? input * 1000 : Date.parse(input);
  if (!Number.isFinite(t)) return "—";
  const s = Math.max(0, (Date.now() - t) / 1000);
  if (s < 60) return "just now";
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ago`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ago`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d ago`;
  const w = Math.floor(d / 7);
  if (w < 5) return `${w}w ago`;
  const mo = Math.floor(d / 30);
  if (mo < 12) return `${mo}mo ago`;
  const y = Math.floor(d / 365);
  return `${y}y ago`;
}
