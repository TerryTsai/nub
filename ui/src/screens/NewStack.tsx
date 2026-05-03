import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";

const NAME_RE = /^[a-z0-9_-]+$/;

const PLACEHOLDER = `services:
  app:
    image: nginx:1.27
    ports:
      - "8080:80"
`;

export function NewStack() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [name, setName] = useState("");
  const [yaml, setYaml] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new stack" }];
  const canCreate = session.session?.can("stacks:create") ?? false;
  const denyCreate = !canCreate ? "your token doesn't allow stacks:create" : undefined;

  async function onCreate() {
    if (!host) return;
    const trimmed = name.trim();
    if (!NAME_RE.test(trimmed) || trimmed.length > 63) {
      setError("name must be lowercase alphanumeric, `-`, or `_`, 1–63 chars");
      return;
    }
    if (yaml.trim() === "") {
      setError("paste a compose YAML before creating");
      return;
    }
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "create_stack", name: trimmed, yaml }), "stack_created");
      invalidate(`${host.url}:list_stacks`);
      invalidate(`${host.url}:list_containers`);
      nav(`/h/${hid}/stacks/${encodeURIComponent(trimmed)}`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  return (
    <Page crumbs={crumbs}>
      <Heading category="Stack" title="new stack" />

      <Section label="Name">
        <Field label="Stack name" hint="lowercase letters, digits, dash, underscore">
          <input
            className="input mono"
            type="text"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            placeholder="myapp"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
        </Field>
      </Section>

      <Section label="Compose YAML">
        <textarea
          className="input mono"
          spellCheck={false}
          autoCapitalize="off"
          autoCorrect="off"
          rows={18}
          placeholder={PLACEHOLDER}
          value={yaml}
          onChange={(e) => setYaml(e.target.value)}
          style={{ minHeight: "320px", whiteSpace: "pre", overflowWrap: "normal", overflowX: "auto" }}
        />
      </Section>

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <Section label="Actions">
        <div className="flex gap-2">
          <Button variant="ghost" onClick={() => nav(`/h/${hid}/stacks`)} className="flex-1">
            Cancel
          </Button>
          <Button onClick={onCreate} disabled={pending} disallowReason={denyCreate} className="flex-1">
            {pending ? "Creating…" : "Create"}
          </Button>
        </div>
      </Section>
    </Page>
  );
}
