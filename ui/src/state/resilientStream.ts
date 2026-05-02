import { useEffect, useRef, useState } from "react";
import { streamOp, type Host } from "@/api/client";
import type { Op, StreamChunk } from "@/api/types";

export type ConnState = "idle" | "streaming" | "reconnecting";

const BACKOFF_MS = [1000, 2000, 5000, 10000, 30000];

/** Run a streaming op and auto-reconnect on unexpected disconnect.
 *
 * Reconnects on rejection (websocket close, engine err response). Stops
 * on clean stream end (`end` with ok=true) — the container is gone, so
 * retrying would hammer for nothing. Stops on signal abort.
 *
 * Backoff: 1s, 2s, 5s, 10s, then 30s. Caller is expected to render the
 * `state` somewhere visible so reconnects are not silent. */
export function useResilientStream(
  host: Host | undefined,
  op: Op | null,
  onChunk: (chunk: StreamChunk) => void,
): { state: ConnState; error: string | null } {
  const [state, setState] = useState<ConnState>("idle");
  const [error, setError] = useState<string | null>(null);
  const onChunkRef = useRef(onChunk);
  onChunkRef.current = onChunk;

  useEffect(() => {
    if (!host || !op) {
      setState("idle");
      return;
    }
    const controller = new AbortController();
    let attempt = 0;
    setError(null);

    (async () => {
      let resolvedCleanly = false;
      while (!controller.signal.aborted && !resolvedCleanly) {
        setState(attempt === 0 ? "streaming" : "reconnecting");
        try {
          await streamOp(host, op, (c) => onChunkRef.current(c), controller.signal);
          resolvedCleanly = true;
        } catch (e) {
          if (controller.signal.aborted) break;
          setError((e as Error).message);
        }
        if (controller.signal.aborted || resolvedCleanly) break;
        const delay = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)];
        attempt += 1;
        await sleep(delay, controller.signal);
      }
      if (!controller.signal.aborted) setState("idle");
    })();

    return () => controller.abort();
    // op is structurally compared via JSON; callers that pass new objects
    // each render would loop forever otherwise.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [host?.url, host?.token, JSON.stringify(op)]);

  return { state, error };
}

function sleep(ms: number, signal: AbortSignal): Promise<void> {
  return new Promise((resolve) => {
    if (signal.aborted) return resolve();
    const t = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => {
      clearTimeout(t);
      resolve();
    });
  });
}
