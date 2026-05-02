import type { Frame, Op, OpResult, StreamChunk } from "./types";

export interface Host {
  /** Base URL: e.g. http://127.0.0.1:8080 (no trailing slash, no /api). */
  url: string;
  /** Bearer token for this host. */
  token: string;
}

export class ApiError extends Error {
  constructor(public status: number, public body: string) {
    super(`HTTP ${status}: ${body}`);
  }
}

/** Send a unary op via POST /api/op. Returns the OpResult. */
export async function call(host: Host, op: Op): Promise<OpResult> {
  const res = await fetch(`${host.url}/api/op`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${host.token}`,
    },
    body: JSON.stringify(op),
  });
  if (!res.ok) {
    throw new ApiError(res.status, await res.text());
  }
  return (await res.json()) as OpResult;
}

/**
 * Narrow an OpResult to a specific success variant. Throws with the engine's
 * message on `err`, or a generic mismatch message on any other type. Without
 * this, every call site has to remember to check `err` before checking the
 * expected type — and forgetting surfaces "unexpected response" when the real
 * cause is e.g. a broken engine.
 */
export function unwrap<T extends OpResult["type"]>(
  r: OpResult,
  expected: T,
): Extract<OpResult, { type: T }> {
  if (r.type === "err") throw new Error(r.data.message);
  if (r.type !== expected) throw new Error(`unexpected response type ${r.type}`);
  return r as Extract<OpResult, { type: T }>;
}

/**
 * Run a streaming op over WebSocket. `onChunk` fires for every stream frame;
 * the returned promise resolves on `end` (ok=true) or rejects on `end` (ok=false),
 * server error response, or socket failure.
 */
export function streamOp(
  host: Host,
  op: Op,
  onChunk: (chunk: StreamChunk) => void,
): Promise<void> {
  const wsUrl = host.url.replace(/^http/, "ws") + "/api/ws";
  // Two subprotocols: "nub" is what the server echoes back; "bearer.<token>"
  // smuggles the token (browsers can't set Authorization on `new WebSocket()`).
  const ws = new WebSocket(wsUrl, ["nub", `bearer.${host.token}`]);
  return new Promise((resolve, reject) => {
    let started = false;
    ws.onopen = () => ws.send(JSON.stringify({ kind: "request", id: 1, op }));
    ws.onerror = () => reject(new Error("websocket error"));
    ws.onclose = () => {
      if (!started) reject(new Error("websocket closed before stream started"));
    };
    ws.onmessage = (e) => {
      const f = JSON.parse(e.data as string) as Frame;
      if (f.kind === "response") {
        if (f.result.type === "err") {
          ws.close();
          reject(new Error(f.result.data.message));
        } else if (f.result.type === "stream_started") {
          started = true;
        } else {
          ws.close();
          reject(new Error(`unexpected response type ${f.result.type}`));
        }
      } else if (f.kind === "stream") {
        if (f.chunk.type === "end") {
          ws.close();
          if (f.chunk.ok) resolve();
          else reject(new Error(f.chunk.err ?? "stream ended with error"));
        } else {
          onChunk(f.chunk);
        }
      }
    };
  });
}
