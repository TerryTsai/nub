import { useEffect, useState } from "react";
import { call, unwrap, type Host } from "@/api/client";
import type { WhoamiInfo } from "@/api/types";

export interface Session {
  whoami: WhoamiInfo;
  /** True if this caller can invoke the given op name. */
  can(op: string): boolean;
}

interface State {
  loading: boolean;
  session: Session | null;
  error: string | null;
}

/** Loads the caller's identity + permissions for a host. Re-runs on host change. */
export function useSession(host: Host | undefined): State {
  const [state, setState] = useState<State>({ loading: true, session: null, error: null });

  useEffect(() => {
    if (!host) {
      setState({ loading: false, session: null, error: null });
      return;
    }
    let cancelled = false;
    setState({ loading: true, session: null, error: null });
    call(host, { op: "whoami" })
      .then((r) => {
        if (cancelled) return;
        const w = unwrap(r, "whoami");
        const allowed = new Set(w.data.allowed);
        const can = (op: string) => allowed.has("*") || allowed.has(op);
        setState({ loading: false, session: { whoami: w.data, can }, error: null });
      })
      .catch((e: Error) => {
        if (cancelled) return;
        setState({ loading: false, session: null, error: e.message });
      });
    return () => {
      cancelled = true;
    };
  }, [host?.url, host?.token]);

  return state;
}
