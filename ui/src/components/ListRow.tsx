import type { ReactNode } from "react";
import type { Status } from "@/state/status";
import { StatusBadge } from "./StatusBadge";

interface Props {
  title: ReactNode;
  subtitle?: ReactNode;
  status?: Status;
  /** Render the title in mono (for ids/hashes). */
  mono?: boolean;
  onPress?: () => void;
  /** Escape hatch for non-entity rows (e.g. saved hosts on the home screen).
   * For container/image/volume/network rows, prefer `status` — actions
   * belong on the detail page, not inline. */
  right?: ReactNode;
}

/** A row in a vertical list. Foundry-style: title + muted subtitle + a
 * status badge aligned right. The right slot is reserved for status on
 * entity lists — actions live on the detail page. */
export function ListRow({ title, subtitle, status, mono, onPress, right }: Props) {
  const cls =
    "w-full text-left border-b border-[var(--border-subtle)] py-3.5 flex items-center gap-3 active:bg-zinc-900/60 transition-colors";
  const inner = (
    <>
      <div className="flex-1 min-w-0">
        <div className={`leading-snug truncate ${mono ? "mono text-xs" : "text-sm"}`}>{title}</div>
        {subtitle && (
          <div className="text-xs text-[var(--text-tertiary)] mt-0.5 line-clamp-2">{subtitle}</div>
        )}
      </div>
      {status && <StatusBadge status={status} />}
      {right}
    </>
  );
  if (onPress && right) {
    return (
      <div
        role="button"
        tabIndex={0}
        onClick={onPress}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            onPress();
          }
        }}
        className={cls}
      >
        {inner}
      </div>
    );
  }
  if (onPress) {
    return (
      <button type="button" onClick={onPress} className={cls}>
        {inner}
      </button>
    );
  }
  return <div className={cls}>{inner}</div>;
}
