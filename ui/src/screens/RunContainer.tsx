import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { SEEDED_TEMPLATES, type Template } from "@/state/templates";
import type { PortPublish, RestartPolicySpec } from "@/api/types";
import { Button } from "@/components/Button";
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";

type LayerProgress = { status: string; current: number; total: number };
interface PullState {
  layers: Record<string, LayerProgress>;
  lastStatus: string;
}

interface FormState {
  image: string;
  name: string;
  ports: PortPublish[];
  env: string[];
  restart: RestartPolicySpec;
  start: boolean;
}

const EMPTY_FORM: FormState = {
  image: "",
  name: "",
  ports: [],
  env: [],
  restart: { kind: "unless_stopped" },
  start: true,
};

export function RunContainer() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pull, setPull] = useState<PullState | null>(null);

  const denyReason =
    session.session && !session.session.can("create_container")
      ? "your token doesn't allow create_container"
      : undefined;

  function applyTemplate(t: Template) {
    setForm({
      image: t.image,
      name: t.containerName ?? "",
      ports: [...t.ports],
      env: [...t.env],
      restart: t.restart ?? { kind: "unless_stopped" },
      start: true,
    });
    setError(null);
  }

  async function onRun(e: React.FormEvent) {
    if (!host) return;
    e.preventDefault();
    setPending(true);
    setError(null);
    const image = form.image.trim();
    try {
      // Pull first — Podman's compat API doesn't auto-pull on create, and
      // showing progress is better UX than a silent hang.
      setPull({ layers: {}, lastStatus: "starting pull…" });
      await streamOp(host, { op: "pull_image", reference: image }, (chunk) => {
        if (chunk.type !== "pull_progress") return;
        setPull((prev) => {
          const layers = { ...(prev?.layers ?? {}) };
          if (chunk.id) {
            layers[chunk.id] = { status: chunk.status, current: chunk.current, total: chunk.total };
          }
          return { layers, lastStatus: chunk.status || prev?.lastStatus || "" };
        });
      });
      const r = unwrap(
        await call(host, {
          op: "create_container",
          image,
          name: form.name.trim() || undefined,
          ports: form.ports.length ? form.ports : undefined,
          env: form.env.length ? form.env : undefined,
          restart: form.restart,
          start: form.start,
        }),
        "container_created",
      );
      nav(`/h/${hid}/c/${r.data.id}`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
      setPull(null);
    } finally {
      setPending(false);
    }
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) {
    return <Page><p>Unknown host.</p></Page>;
  }

  const crumbs: Crumb[] = [hostCrumb, { kind: "link", label: "new container" }];

  return (
    <Page crumbs={crumbs}>
      <Heading category="Run" title="New container" />

      {denyReason && <p className="text-[var(--warn)] text-xs">{denyReason}</p>}

      <Section label="Templates">
        <div className="-mx-5 px-5 overflow-x-auto">
          <div className="flex gap-2 pb-1">
            {SEEDED_TEMPLATES.map((t) => (
              <button
                key={t.id}
                type="button"
                onClick={() => applyTemplate(t)}
                className="template-chip"
              >
                <div className="text-xs font-medium">{t.name}</div>
                <div className="text-[11px] text-[var(--text-tertiary)] mono truncate">{t.image}</div>
              </button>
            ))}
          </div>
        </div>
      </Section>

      <form onSubmit={onRun} className="contents">
        <Section label="Container">
          <Field label="Image" hint="e.g. nginx:alpine, postgres:16, ghcr.io/...">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="image:tag"
              value={form.image}
              onChange={(e) => setForm({ ...form, image: e.target.value })}
              required
            />
          </Field>
          <Field label="Name" hint="optional — engine auto-names if blank">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="my-app"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
            />
          </Field>
        </Section>

        <Section label="Ports">
          <PairList
            pairs={form.ports.map((p) => [p.container, p.host])}
            placeholder={["80/tcp", "8080"]}
            onChange={(rows) =>
              setForm({ ...form, ports: rows.map(([container, host]) => ({ container, host })) })
            }
          />
        </Section>

        <Section label="Environment">
          <StringList
            values={form.env}
            placeholder="KEY=value"
            onChange={(env) => setForm({ ...form, env })}
          />
        </Section>

        <Section label="Restart policy">
          <RestartPicker value={form.restart} onChange={(restart) => setForm({ ...form, restart })} />
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={form.start}
              onChange={(e) => setForm({ ...form, start: e.target.checked })}
            />
            <span className="text-xs">Start after create</span>
          </label>
        </Section>

        {pull && (
          <Section label="Pull progress">
            <PullProgress pull={pull} />
          </Section>
        )}

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="Actions">
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav(`/h/${hid}`)} className="flex-1">
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !form.image.trim()}
              disallowReason={denyReason}
              className="flex-1"
            >
              {pending ? "…" : "Run"}
            </Button>
          </div>
        </Section>
      </form>
    </Page>
  );
}

function PairList({
  pairs,
  placeholder,
  onChange,
}: {
  pairs: [string, string][];
  placeholder: [string, string];
  onChange: (rows: [string, string][]) => void;
}) {
  function set(i: number, j: 0 | 1, v: string) {
    const next = pairs.map((p) => [...p] as [string, string]);
    next[i][j] = v;
    onChange(next);
  }
  return (
    <div className="flex flex-col gap-2">
      {pairs.map((p, i) => (
        <div key={i} className="flex gap-2">
          <input
            className="input mono flex-1"
            value={p[0]}
            placeholder={placeholder[0]}
            onChange={(e) => set(i, 0, e.target.value)}
          />
          <span className="self-center text-[var(--text-tertiary)]">→</span>
          <input
            className="input mono flex-1"
            value={p[1]}
            placeholder={placeholder[1]}
            onChange={(e) => set(i, 1, e.target.value)}
          />
          <Button variant="ghost" onClick={() => onChange(pairs.filter((_, j) => j !== i))}>
            ×
          </Button>
        </div>
      ))}
      <Button variant="ghost" onClick={() => onChange([...pairs, ["", ""]])} className="self-start">
        + Add port
      </Button>
    </div>
  );
}

function StringList({
  values,
  placeholder,
  onChange,
}: {
  values: string[];
  placeholder: string;
  onChange: (vs: string[]) => void;
}) {
  return (
    <div className="flex flex-col gap-2">
      {values.map((v, i) => (
        <div key={i} className="flex gap-2">
          <input
            className="input mono flex-1"
            value={v}
            placeholder={placeholder}
            onChange={(e) => onChange(values.map((x, j) => (j === i ? e.target.value : x)))}
          />
          <Button variant="ghost" onClick={() => onChange(values.filter((_, j) => j !== i))}>
            ×
          </Button>
        </div>
      ))}
      <Button variant="ghost" onClick={() => onChange([...values, ""])} className="self-start">
        + Add row
      </Button>
    </div>
  );
}

function RestartPicker({
  value,
  onChange,
}: {
  value: RestartPolicySpec;
  onChange: (v: RestartPolicySpec) => void;
}) {
  const opts: { kind: RestartPolicySpec["kind"]; label: string }[] = [
    { kind: "no", label: "No" },
    { kind: "on_failure", label: "On failure" },
    { kind: "always", label: "Always" },
    { kind: "unless_stopped", label: "Unless stopped" },
  ];
  return (
    <div className="grid grid-cols-2 gap-2">
      {opts.map((o) => {
        const active = value.kind === o.kind;
        return (
          <button
            key={o.kind}
            type="button"
            onClick={() => onChange({ kind: o.kind } as RestartPolicySpec)}
            className={`btn ${active ? "btn-primary" : "btn-ghost"}`}
          >
            {o.label}
          </button>
        );
      })}
    </div>
  );
}

function PullProgress({ pull }: { pull: PullState }) {
  const layers = Object.entries(pull.layers);
  return (
    <div className="text-xs flex flex-col gap-2">
      <div className="text-[var(--text-secondary)]">{pull.lastStatus || "pulling…"}</div>
      {layers.length > 0 && (
        <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
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
