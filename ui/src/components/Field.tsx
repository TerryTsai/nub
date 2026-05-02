import type { ReactNode } from "react";

/** Form field: small caps label, control, optional muted hint. */
export function Field({
  label,
  children,
  hint,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
}) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
        {label}
      </span>
      {children}
      {hint && <span className="text-[11px] text-[var(--text-tertiary)]">{hint}</span>}
    </label>
  );
}
