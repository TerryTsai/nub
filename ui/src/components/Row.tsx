import type { ReactNode } from "react";

/** Key/value row used inside a Section. Mono values get amber treatment to
 * match foundry's "ID-as-link" convention — use `mono` for identifiers /
 * paths / timestamps; omit it for prose. */
export function Row({
  label,
  value,
  mono,
  right,
}: {
  label: string;
  value?: string;
  mono?: boolean;
  right?: ReactNode;
}) {
  return (
    <div className="flex gap-3 items-baseline select-text">
      <span className="text-xs text-[var(--text-tertiary)] shrink-0 w-24">{label}</span>
      {right ? (
        <div className="flex-1 min-w-0">{right}</div>
      ) : (
        <span
          className={`flex-1 min-w-0 text-xs leading-5 break-all ${
            mono ? "mono text-[var(--id-color)]" : "text-[var(--text-primary)]"
          }`}
        >
          {value}
        </span>
      )}
    </div>
  );
}
