import { useState, type ReactNode } from "react";

interface Props {
  /** Small-caps label, mirrors Section. Defaults to `info`. */
  label?: string;
  /** Start expanded? Default false — info groups stay out of the way. */
  defaultOpen?: boolean;
  children: ReactNode;
  className?: string;
}

/** Folded info group — a `<details>`-shaped section that matches the
 * visual treatment of `Section` (small-caps label, top border, gap-2
 * children). Use to stash explanatory copy that's useful but doesn't
 * need to be on screen by default.
 *
 * Built on the native `<details>` so keyboard expand and screen-reader
 * semantics come for free; CSS hides the default disclosure marker so
 * we can render our own chevron beside the label. */
export function Collapsible({ label = "info", defaultOpen = false, children, className = "" }: Props) {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <details
      open={open}
      onToggle={(e) => setOpen((e.currentTarget as HTMLDetailsElement).open)}
      className={`pt-3 border-t border-[var(--border-subtle)] first:border-t-0 first:pt-0 ${className}`}
    >
      <summary
        className="flex items-center gap-1 cursor-pointer list-none select-none text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]"
      >
        <Chevron open={open} />
        <span>{label}</span>
      </summary>
      <div className="flex flex-col gap-2 mt-2">{children}</div>
    </details>
  );
}

function Chevron({ open }: { open: boolean }) {
  return (
    <svg
      width="10"
      height="10"
      viewBox="0 0 10 10"
      aria-hidden="true"
      className="shrink-0 transition-transform duration-150"
      style={{ transform: open ? "rotate(90deg)" : "rotate(0deg)" }}
    >
      <path d="M3.5 2l3 3-3 3" stroke="currentColor" fill="none" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}
