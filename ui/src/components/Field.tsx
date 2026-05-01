import type { ReactNode } from "react";

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
    <label className="flex flex-col gap-1.5 px-0.5">
      <span className="text-xs font-medium uppercase tracking-wider text-[var(--text-tertiary)]">
        {label}
      </span>
      {children}
      {hint && <span className="text-xs text-[var(--text-tertiary)]">{hint}</span>}
    </label>
  );
}
