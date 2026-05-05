import type { ReactNode } from "react";
import type { ConnState } from "@/state/resilientStream";

type Tone = "active" | "pending" | "failed" | "idle";

/** Inline status strip — colored dot + short label. The home for transient
 * state that doesn't fit elsewhere: a stream connecting/reconnecting, an
 * inflight load on a non-list page, a save in progress on a sub-screen.
 *
 * Lives in the page body (subnav is reserved for filters/actions; toasts
 * auto-dismiss). Callers position it where it makes sense — typically as
 * the first child of the body. */
export function StatusLine({
  tone = "pending",
  children,
}: {
  tone?: Tone;
  children: ReactNode;
}) {
  return (
    <div className="flex items-center gap-2 px-5 py-1.5 border-b border-[var(--border-subtle)]">
      <span className={`dot dot-${tone}`} aria-hidden="true" />
      <span className="text-[11px] text-[var(--text-secondary)]">{children}</span>
    </div>
  );
}

/** ConnState → StatusLine. Only surfaces during reconnect; healthy
 * streaming and idle are both quiet. Use a plain StatusLine for the
 * "no data yet" case — that's per-screen state, not connection state. */
export function StreamStatus({ state }: { state: ConnState }) {
  if (state === "reconnecting") return <StatusLine tone="pending">Reconnecting…</StatusLine>;
  return null;
}
