/** ARG declaration parsed from a Dockerfile.
 *
 * Lightweight: line-based, recognizes `ARG NAME` and `ARG NAME=DEFAULT`.
 * Multi-line ARG (line continuations) is not handled — vanishingly rare
 * in the wild and the build will surface a clearer error if it matters. */
export interface DockerfileArg {
  name: string;
  default?: string;
}

const ARG_LINE = /^\s*ARG\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*=\s*(.*))?$/;

export function parseArgs(content: string): DockerfileArg[] {
  const out: DockerfileArg[] = [];
  for (const raw of content.split("\n")) {
    const line = raw.replace(/\r$/, "");
    if (!line.trim() || line.trim().startsWith("#")) continue;
    const m = ARG_LINE.exec(line);
    if (!m) continue;
    const name = m[1];
    let value = m[2]?.trim();
    if (value !== undefined) {
      // Strip surrounding quotes if present (single or double, matching).
      if (
        value.length >= 2 &&
        ((value.startsWith('"') && value.endsWith('"')) ||
          (value.startsWith("'") && value.endsWith("'")))
      ) {
        value = value.slice(1, -1);
      }
    }
    out.push({ name, default: value });
  }
  return out;
}
