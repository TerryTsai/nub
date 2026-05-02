/** Universal status vocabulary used everywhere in the UI.
 *
 * Tone (color) is consistent across entity types — green is always
 * "engaged," gray is always "idle," amber is always "in flight," red
 * is always "broken." The label is entity-specific but always pairs
 * with the right tone.
 *
 * Add a new entity type here so the mapping stays in one place. */

export type Tone = "active" | "pending" | "failed" | "idle";

export interface Status {
  tone: Tone;
  label: string;
}

export function containerStatus(state: string, exitCode: number): Status {
  switch (state) {
    case "running":    return { tone: "active",  label: "Running" };
    case "paused":     return { tone: "pending", label: "Paused" };
    case "restarting": return { tone: "pending", label: "Restarting" };
    case "removing":   return { tone: "pending", label: "Removing" };
    case "created":    return { tone: "idle",    label: "Created" };
    case "dead":       return { tone: "failed",  label: "Dead" };
    case "exited":
      return exitCode === 0
        ? { tone: "idle",   label: "Stopped" }
        : { tone: "failed", label: "Failed" };
    default:
      return { tone: "idle", label: state || "Unknown" };
  }
}

export function imageStatus(containers: number): Status {
  return containers > 0
    ? { tone: "active", label: "In use" }
    : { tone: "idle",   label: "Idle" };
}

export function volumeStatus(inUse: boolean): Status {
  return inUse
    ? { tone: "active", label: "In use" }
    : { tone: "idle",   label: "Idle" };
}

export function networkStatus(inUse: boolean): Status {
  return inUse
    ? { tone: "active", label: "In use" }
    : { tone: "idle",   label: "Idle" };
}
