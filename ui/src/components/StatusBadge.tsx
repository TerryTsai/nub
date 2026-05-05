import type { Status } from "@/state/status";

/** Small dot + label. Tone (dot color) is universal across entity types;
 * label is entity-specific. Compute via the helpers in `state/status.ts`.
 *
 * Pass `null` while detail data is loading so the badge still occupies its
 * slot — keeps page layout stable when the real status pops in. */
export function StatusBadge({ status }: { status: Status | null }) {
  const tone = status?.tone ?? "idle";
  const label = status?.label ?? "—";
  return (
    <span className="flex items-center gap-1.5 shrink-0">
      <span className={`dot dot-${tone}`} aria-label={label} />
      <span className="text-[11px] text-[var(--text-secondary)]">{label}</span>
    </span>
  );
}
