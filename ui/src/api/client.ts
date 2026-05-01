import type { Op, OpResult } from "./types";

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
