import type { PortPublish, RestartPolicySpec } from "@/api/types";

/** A pre-filled form configuration for the Run sheet. */
export interface Template {
  id: string;
  name: string;
  image: string;
  /** Optional name suggestion for the new container (otherwise Docker auto-names). */
  containerName?: string;
  ports: PortPublish[];
  env: string[];
  restart?: RestartPolicySpec;
  /** Free-form note shown under the chip (e.g. "edit the env first"). */
  note?: string;
}

/** Seeded templates. Editable in a future slice; for now they're starting points. */
export const SEEDED_TEMPLATES: Template[] = [
  {
    id: "postgres",
    name: "Postgres",
    image: "postgres:16",
    ports: [{ container: "5432/tcp", host: "5432" }],
    env: ["POSTGRES_PASSWORD=changeme"],
    restart: { kind: "unless_stopped" },
    note: "edit POSTGRES_PASSWORD before running",
  },
  {
    id: "redis",
    name: "Redis",
    image: "redis:7-alpine",
    ports: [{ container: "6379/tcp", host: "6379" }],
    env: [],
    restart: { kind: "unless_stopped" },
  },
  {
    id: "nginx",
    name: "nginx",
    image: "nginx:alpine",
    ports: [{ container: "80/tcp", host: "8080" }],
    env: [],
    restart: { kind: "unless_stopped" },
  },
  {
    id: "alpine-shell",
    name: "alpine shell",
    image: "alpine:latest",
    ports: [],
    env: [],
    note: "exec into it after creating",
  },
  {
    id: "claude-code",
    name: "Claude Code",
    image: "node:22-bookworm-slim",
    ports: [],
    env: ["ANTHROPIC_API_KEY="],
    note: "starting point — set ANTHROPIC_API_KEY and customize",
  },
];
