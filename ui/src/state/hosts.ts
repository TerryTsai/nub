import { useEffect, useState } from "react";

export interface SavedHost {
  /** Stable client-side identifier, opaque (uuid). */
  hid: string;
  /** Display name (defaults to host's reported id, can be edited). */
  label: string;
  /** Base URL: e.g. http://10.0.0.5:8080 */
  url: string;
  /** Bearer token. */
  token: string;
}

const KEY = "nub:hosts:v1";

interface Stored {
  version: 1;
  hosts: SavedHost[];
}

function load(): SavedHost[] {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as Stored;
    if (parsed.version !== 1 || !Array.isArray(parsed.hosts)) return [];
    return parsed.hosts;
  } catch {
    return [];
  }
}

function save(hosts: SavedHost[]) {
  const v: Stored = { version: 1, hosts };
  localStorage.setItem(KEY, JSON.stringify(v));
}

export function newId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) return crypto.randomUUID();
  return Math.random().toString(36).slice(2) + Date.now().toString(36);
}

/** React hook over the saved hosts list. */
export function useHosts() {
  const [hosts, setHosts] = useState<SavedHost[]>(() => load());

  useEffect(() => save(hosts), [hosts]);

  return {
    hosts,
    add: (h: Omit<SavedHost, "hid">) => {
      const entry: SavedHost = { hid: newId(), ...h };
      setHosts((prev) => [...prev, entry]);
      return entry;
    },
    update: (hid: string, patch: Partial<Omit<SavedHost, "hid">>) =>
      setHosts((prev) => prev.map((h) => (h.hid === hid ? { ...h, ...patch } : h))),
    remove: (hid: string) => setHosts((prev) => prev.filter((h) => h.hid !== hid)),
    get: (hid: string) => hosts.find((h) => h.hid === hid),
  };
}
