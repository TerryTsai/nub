import { call, unwrap, type Host } from "@/api/client";
import type { ContainerDetail, ContainerSummary } from "@/api/types";
import { peek, useQuery } from "@/state/cache";

/** Best-effort container name across cache layers, in order:
 *  1. inspect:cid cache (full detail name) — primed by ContainerDetail.
 *  2. list_containers cache — primed by HostHome (the most common entry).
 *  3. cid prefix — only on cold direct-link load.
 *
 * Used by Logs / Stats / Exec / Detail crumbs to avoid the hex-then-name
 * swap when navigating from a list. */
export function useContainerName(host: Host | undefined, cid: string | undefined): string {
  const key = host && cid ? `${host.url}:inspect:${cid}` : null;
  const { data } = useQuery<ContainerDetail>(key, async () => {
    const r = unwrap(await call(host!, { op: "get_container", id: cid! }), "container_detail");
    return r.data;
  });
  if (data?.name) return data.name;
  if (host && cid) {
    const list = peek<ContainerSummary[]>(`${host.url}:list_containers`);
    const hit = list?.find((c) => c.id === cid);
    if (hit?.name) return hit.name;
  }
  return cid?.slice(0, 12) || "?";
}
