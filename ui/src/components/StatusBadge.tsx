import type { Status } from "@/state/status";

/** Small dot + label. Tone (dot color) is universal across entity types;
 * label is entity-specific. Compute via the helpers in `state/status.ts`. */
export function StatusBadge({ status }: { status: Status }) {
  return (
    <span className="flex items-center gap-1.5 shrink-0">
      <span className={`dot dot-${status.tone}`} aria-label={status.label} />
      <span className="text-[11px] text-[var(--text-secondary)]">{status.label}</span>
    </span>
  );
}
