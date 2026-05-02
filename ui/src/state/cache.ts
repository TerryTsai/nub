import { useEffect, useState } from "react";

/** Stale-while-revalidate cache for list-style queries.
 *
 * On mount, return any previous value for `key` synchronously so the page
 * doesn't paint a "Loading…" placeholder; fire `fetcher()` in the
 * background and update the cache when it lands. Multiple components on
 * the same key share one in-flight request and re-render together.
 *
 * No invalidation policy — entries live until the page reloads. That's
 * fine: list pages call this once per mount, the most recent fetch wins,
 * and Refresh is always one tap away.
 */

interface Entry<T> {
  data: T | null;
  error: string | null;
  /** A promise of the in-flight refresh, if one is happening. Used to
   * dedupe concurrent fetches on the same key. */
  pending: Promise<void> | null;
}

const cache = new Map<string, Entry<unknown>>();
const subs = new Map<string, Set<() => void>>();

function notify(key: string) {
  subs.get(key)?.forEach((cb) => cb());
}

function read<T>(key: string): Entry<T> {
  return (cache.get(key) as Entry<T>) ?? { data: null, error: null, pending: null };
}

function write<T>(key: string, e: Entry<T>) {
  cache.set(key, e as Entry<unknown>);
  notify(key);
}

async function refresh<T>(key: string, fetcher: () => Promise<T>): Promise<void> {
  const current = read<T>(key);
  if (current.pending) return current.pending;
  const promise = (async () => {
    try {
      const data = await fetcher();
      write<T>(key, { data, error: null, pending: null });
    } catch (e) {
      write<T>(key, { data: read<T>(key).data, error: (e as Error).message, pending: null });
    }
  })();
  cache.set(key, { ...current, pending: promise } as Entry<unknown>);
  return promise;
}

export interface QueryResult<T> {
  data: T | null;
  error: string | null;
  /** True while a fetch is in-flight (initial load OR background refresh). */
  refreshing: boolean;
  /** Force a re-fetch. */
  reload: () => void;
}

/** Subscribe to `key`'s cache slot and trigger a refresh. Cached value is
 * returned synchronously; the fetch runs in the background and updates
 * subscribers when it completes. */
export function useQuery<T>(
  key: string | null,
  fetcher: () => Promise<T>,
): QueryResult<T> {
  const [, setTick] = useState(0);

  useEffect(() => {
    if (!key) return;
    const set = subs.get(key) ?? new Set();
    const cb = () => setTick((t) => t + 1);
    set.add(cb);
    subs.set(key, set);
    void refresh(key, fetcher);
    return () => {
      set.delete(cb);
      if (set.size === 0) subs.delete(key);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key]);

  if (!key) return { data: null, error: null, refreshing: false, reload: () => {} };
  const entry = read<T>(key);
  return {
    data: entry.data,
    error: entry.error,
    refreshing: entry.pending !== null,
    reload: () => void refresh(key, fetcher),
  };
}

/** Drop one cache entry — useful after a mutating op (e.g., remove image)
 * so the next visit pulls fresh. Doesn't trigger a refetch on its own. */
export function invalidate(key: string) {
  cache.delete(key);
  notify(key);
}

/** Read a cache entry without subscribing. Used by detail pages to seed a
 * placeholder name from the parent list before the detail fetch lands. */
export function peek<T>(key: string): T | null {
  return read<T>(key).data;
}
