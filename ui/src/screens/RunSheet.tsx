import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import { SEEDED_TEMPLATES, type Template } from "@/state/templates";
import type { PortPublish, RestartPolicySpec } from "@/api/types";
import { Button } from "@/components/Button";
import { Field } from "@/components/Field";

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

export function RunSheet({
  host,
  open,
  onOpenChange,
  onCreated,
  disallowReason,
}: {
  host: Host;
  open: boolean;
  onOpenChange: (o: boolean) => void;
  onCreated: (id: string) => void;
  disallowReason?: string;
}) {
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pull, setPull] = useState<PullState | null>(null);

  function reset() {
    setForm(EMPTY_FORM);
    setError(null);
    setPull(null);
  }

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
    e.preventDefault();
    setPending(true);
    setError(null);
    const image = form.image.trim();
    try {
      // Pull first — Podman's compat API doesn't auto-pull on create like
      // dockerd does, and showing progress is better UX than a silent hang.
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
      const r = unwrap(await call(host, {
        op: "create_container",
        image,
        name: form.name.trim() || undefined,
        ports: form.ports.length ? form.ports : undefined,
        env: form.env.length ? form.env : undefined,
        restart: form.restart,
        start: form.start,
      }), "container_created");
      onOpenChange(false);
      reset();
      onCreated(r.data.id);
    } catch (e) {
      setError((e as Error).message);
      setPull(null);
    } finally {
      setPending(false);
    }
  }

  return (
    <Dialog.Root open={open} onOpenChange={(o) => { if (!o) reset(); onOpenChange(o); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="sheet-overlay" />
        <Dialog.Content className="sheet glass-strong">
          <div className="sheet-handle" />
          <Dialog.Title className="text-xl font-semibold px-1">Run a container</Dialog.Title>
          <Dialog.Description className="sr-only">
            Pick a template or fill the form to create a new container.
          </Dialog.Description>

          {disallowReason && (
            <p className="text-sm text-[var(--warn)] px-1">{disallowReason}</p>
          )}

          {/* Templates */}
          <div className="-mx-5 px-5 overflow-x-auto">
            <div className="flex gap-2 pb-1">
              {SEEDED_TEMPLATES.map((t) => (
                <button
                  key={t.id}
                  type="button"
                  onClick={() => applyTemplate(t)}
                  className="template-chip"
                >
                  <div className="font-medium">{t.name}</div>
                  <div className="text-xs text-[var(--text-tertiary)] mono truncate">{t.image}</div>
                </button>
              ))}
            </div>
          </div>

          <form onSubmit={onRun} className="flex flex-col gap-3">
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

            <Field label="Name" hint="optional — Docker auto-names if blank">
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

            <Field label="Ports" hint="container → host. e.g. 80/tcp → 8080">
              <PairList
                pairs={form.ports.map((p) => [p.container, p.host])}
                placeholder={["80/tcp", "8080"]}
                onChange={(rows) =>
                  setForm({ ...form, ports: rows.map(([container, host]) => ({ container, host })) })
                }
              />
            </Field>

            <Field label="Environment" hint="one KEY=value per row">
              <StringList
                values={form.env}
                placeholder="KEY=value"
                onChange={(env) => setForm({ ...form, env })}
              />
            </Field>

            <Field label="Restart policy">
              <RestartPicker
                value={form.restart}
                onChange={(restart) => setForm({ ...form, restart })}
              />
            </Field>

            <label className="flex items-center gap-2 px-1 cursor-pointer">
              <input
                type="checkbox"
                checked={form.start}
                onChange={(e) => setForm({ ...form, start: e.target.checked })}
              />
              <span className="text-sm">Start after create</span>
            </label>

            {pull && <PullProgress pull={pull} />}

            {error && <div className="text-[var(--error)] text-sm px-1">{error}</div>}

            <div className="flex gap-2 mt-1">
              <Button variant="ghost" onClick={() => onOpenChange(false)} className="flex-1">
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={pending || !form.image.trim()}
                disallowReason={disallowReason}
                className="flex-1"
              >
                {pending ? "…" : "Run"}
              </Button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
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
      <Button variant="ghost" onClick={() => onChange([...pairs, ["", ""]])}>
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
      <Button variant="ghost" onClick={() => onChange([...values, ""])}>
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
            className={`btn ${active ? "btn-primary" : "btn-ghost"} text-sm`}
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
    <div className="px-1 text-sm flex flex-col gap-2">
      <div className="text-[var(--text-secondary)]">{pull.lastStatus || "pulling…"}</div>
      {layers.length > 0 && (
        <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
          {layers.map(([id, p]) => {
            const pct = p.total > 0 ? Math.min(100, Math.round((p.current / p.total) * 100)) : null;
            return (
              <div key={id} className="flex items-center gap-2 text-xs">
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
