import type { StreamChunk } from "@/api/types";

export interface PullState {
  layers: Record<string, { status: string; current: number; total: number }>;
  lastStatus: string;
}

export const EMPTY_PULL: PullState = { layers: {}, lastStatus: "" };

/** Fold one engine pull-progress chunk into a PullState. Caller is responsible
 * for storing the result in component state — this is a pure reducer. */
export function reducePull(prev: PullState, chunk: StreamChunk): PullState {
  if (chunk.type !== "pull_progress") return prev;
  const layers = { ...prev.layers };
  if (chunk.id) {
    layers[chunk.id] = { status: chunk.status, current: chunk.current, total: chunk.total };
  }
  return { layers, lastStatus: chunk.status || prev.lastStatus };
}

export function PullProgress({ pull }: { pull: PullState }) {
  const layers = Object.entries(pull.layers);
  return (
    <div className="text-xs flex flex-col gap-2">
      <div className="text-[var(--text-secondary)]">{pull.lastStatus || "pulling…"}</div>
      {layers.length > 0 && (
        <div className="flex flex-col gap-1 max-h-64 overflow-y-auto">
          {layers.map(([id, p]) => {
            const pct = p.total > 0 ? Math.min(100, Math.round((p.current / p.total) * 100)) : null;
            return (
              <div key={id} className="flex items-center gap-2">
                <span className="mono text-[var(--text-tertiary)] w-16 truncate">{id.slice(0, 12)}</span>
                <span className="flex-1 truncate">{p.status}</span>
                {pct !== null && <span className="mono text-[var(--text-tertiary)]">{pct}%</span>}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
