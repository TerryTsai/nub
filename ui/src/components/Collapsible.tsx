import { Children, useState, type ReactNode } from "react";

interface Props {
  /** Small-caps section label. Defaults to `info`. */
  label?: string;
  /** Override the row count badge. When omitted, the count is derived
   * from the number of direct children — fine for fixed-row sections
   * (Container/spec/runtime); list-shaped sections should pass the data
   * length explicitly so empty renders as `0` instead of `1` (the
   * `<EmptyRow />` placeholder). */
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
export function Collapsible(props: Props) {
  const { label = "info", defaultOpen = false, children, className = "" } = props;
  const [open, setOpen] = useState(defaultOpen);
  // Distinguish "count prop omitted" (auto-count children) from "count
  // explicitly undefined" (data still loading — hide the badge).
  const hasExplicitCount = "count" in props;
  const effectiveCount = hasExplicitCount
    ? props.count
    : Children.toArray(children).filter(Boolean).length;
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
        {effectiveCount !== undefined && (
          <span className="ml-auto text-[var(--text-secondary)] font-normal normal-case tracking-normal">
            {effectiveCount}
          </span>
        )}
        <Chevron open={open} className={effectiveCount === undefined ? "ml-auto" : ""} />
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
