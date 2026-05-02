import type { ReactNode } from "react";
import { StatusBadge } from "./StatusBadge";

interface Props {
  title: ReactNode;
  subtitle?: ReactNode;
  status?: string;
  /** Render the title in mono (for ids/hashes). */
  mono?: boolean;
  onPress?: () => void;
  right?: ReactNode;
}

/** A row in a vertical list. Foundry-style: title + muted subtitle + optional
 * status pill aligned right. Border-b separates rows; background hint on
 * press. */
export function ListRow({ title, subtitle, status, mono, onPress, right }: Props) {
  const cls =
    "w-full text-left border-b border-[var(--border-subtle)] py-3.5 flex items-center gap-3 active:bg-zinc-900/60 transition-colors";
  const inner = (
    <>
      <div className="flex-1 min-w-0">
        <div className={`leading-snug truncate ${mono ? "mono text-xs" : "text-sm"}`}>{title}</div>
        {subtitle && (
          <div className="text-xs text-[var(--text-tertiary)] mt-0.5 truncate">{subtitle}</div>
        )}
      </div>
      {right}
      {status && <StatusBadge status={status} />}
    </>
  );
  return onPress ? (
    <button type="button" onClick={onPress} className={cls}>
      {inner}
    </button>
  ) : (
    <div className={cls}>{inner}</div>
  );
}
