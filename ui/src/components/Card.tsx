import type { ReactNode, HTMLAttributes } from "react";

/** Bordered black card. No glass, no rounded blur — flat foundry-style. */
export function Card({
  children,
  className = "",
  ...rest
}: { children: ReactNode } & HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={`border border-[var(--border-subtle)] rounded-[var(--radius-lg)] p-4 flex flex-col gap-3 ${className}`}
      {...rest}
    >
      {children}
    </div>
  );
}

/** Key/value row inside a Card. Mono values get amber treatment to match the
 * foundry "ID" styling. Use `mono` for IDs/hashes; omit it for prose. */
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

/** Section heading inside a Card — small caps, tracked, muted. */
export function SectionLabel({ children }: { children: ReactNode }) {
  return (
    <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
      {children}
    </span>
  );
}
