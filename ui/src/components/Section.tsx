import type { ReactNode } from "react";

interface Props {
  /** Small caps label header for the section. */
  label?: string;
  /** Right-side content next to the label (e.g. action buttons). */
  right?: ReactNode;
  children: ReactNode;
  className?: string;
}

/** Flat section with an optional small-caps header and a top border. The
 * first section in a Page omits its top border via `first:` selector — keeps
 * the page open at the top instead of feeling boxed-in. */
export function Section({ label, right, children, className = "" }: Props) {
  return (
    <section
      className={`flex flex-col gap-2 pt-3 border-t border-[var(--border-subtle)] first:border-t-0 first:pt-0 ${className}`}
    >
      {(label || right) && (
        <div className="flex items-center justify-between gap-2">
          {label && (
            <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
              {label}
            </span>
          )}
          {right}
        </div>
      )}
      {children}
    </section>
  );
}
