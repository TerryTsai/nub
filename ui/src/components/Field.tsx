import type { ReactNode } from "react";

/** Form field: small caps label, control, optional muted hint, optional
 * inline error. The error renders directly under the control in red so the
 * user sees it without scrolling — a single page-level error placement
 * was getting hidden behind the soft keyboard on mobile. */
export function Field({
  label,
  children,
  hint,
  error,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
  error?: string | null;
}) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-[11px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
        {label}
      </span>
      {children}
      {error && <span className="text-[11px] text-[var(--error)]">{error}</span>}
      {!error && hint && <span className="text-[11px] text-[var(--text-tertiary)]">{hint}</span>}
    </label>
  );
}
