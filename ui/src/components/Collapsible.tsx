import { useState, type ReactNode } from "react";

interface Props {
  /** Small-caps section label. Defaults to `info`. */
  label?: string;
  /** Optional count badge after the label, e.g. an env var count. */
  count?: number;
  /** Start expanded? Default false — info groups stay out of the way. */
  defaultOpen?: boolean;
  children: ReactNode;
  className?: string;
}

/** Folded section — a `<details>`-shaped group with a small-caps label
 * header and a chevron. Used as the universal page primitive: the first
 * one on a detail page is `defaultOpen` (the resource's identity), the
 * rest fold closed until the operator wants them.
 *
 * Built on the native `<details>` so keyboard expand and screen-reader
 * semantics come for free; CSS hides the default disclosure marker so
 * we can render our own chevron beside the label. */
export function Collapsible({ label = "info", count, defaultOpen = false, children, className = "" }: Props) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <details
      open={open}
      onToggle={(e) => setOpen((e.currentTarget as HTMLDetailsElement).open)}
      className={className}
    >
      <summary
        className="flex items-center gap-2 cursor-pointer list-none select-none text-[11px] font-semibold uppercase tracking-wider text-[var(--text-primary)]"
      >
        <span>{label}</span>
        {count !== undefined && count > 0 && (
          <span className="ml-auto text-[var(--text-secondary)] font-normal normal-case tracking-normal">
            {count}
          </span>
        )}
        <Chevron open={open} className={count !== undefined && count > 0 ? "" : "ml-auto"} />
      </summary>
      <div className="flex flex-col gap-1.5 mt-1.5">{children}</div>
    </details>
  );
}

function Chevron({ open, className = "" }: { open: boolean; className?: string }) {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      aria-hidden="true"
      className={`shrink-0 transition-transform duration-150 ${className}`}
      style={{ transform: open ? "rotate(90deg)" : "rotate(0deg)" }}
    >
      <path d="M3.5 2l3 3-3 3" stroke="currentColor" fill="none" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
