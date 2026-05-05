import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

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

  const [name, setName] = useState("");
  const [yaml, setYaml] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "stacks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new stack" }];

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "create_stack", name: name.trim(), yaml }), "stack_created");
      invalidate(`${host.url}:list_stacks`);
      invalidate(`${host.url}:list_containers`);
      nav(`/h/${hid}/stacks/${encodeURIComponent(name.trim())}`, { replace: true });
    } catch (e) {
      setError(`create stack: ${(e as Error).message}`);
    } finally {
      setPending(false);
    }
  }

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Stack"
        editable={{
          value: name,
          onChange: setName,
          placeholder: "new stack",
        }}
      />

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Section label="compose">
          <textarea
            className="input-code"
            spellCheck={false}
            autoCapitalize="off"
            autoCorrect="off"
            rows={18}
            placeholder={PLACEHOLDER}
            value={yaml}
            onChange={(e) => setYaml(e.target.value)}
            style={{ minHeight: "320px" }}
          />
        </Section>

        <Section label="create">
          <div className="flex gap-2">
            <Button variant="ghost" onClick={() => nav(`/h/${hid}/stacks`)} className="flex-1">
              Cancel
            </Button>
            <Button type="submit" disabled={pending || !name.trim() || !yaml.trim()} className="flex-1">
              {pending ? (<><Spinner /> Creating…</>) : "Create"}
            </Button>
          </div>
        </Section>
      </form>
    </Page>
  );
}
