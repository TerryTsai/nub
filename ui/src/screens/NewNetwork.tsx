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

const NAME_RE = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;

export function NewNetwork() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [name, setName] = useState("");
  const [internal, setInternal] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const denyReason =
    session.session && !session.session.can("networks:create")
      ? "your token doesn't allow networks:create"
      : undefined;

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const trimmed = name.trim();
    if (!NAME_RE.test(trimmed)) {
      setError("name must start with a letter or digit; letters/digits/dot/underscore/dash only");
      return;
    }
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "create_network", name: trimmed, internal }), "ok");
      invalidate(`${host.url}:list_networks`);
      nav(`/h/${hid}/networks`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new network" }];

  return (
    <Page crumbs={crumbs}>
      <Heading category="Network" title="new network" />

      {denyReason && <p className="text-[var(--warn)] text-xs">{denyReason}</p>}

      <form onSubmit={onSubmit} className="contents">
        <Section label="network">
          <Field label="Name">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="web"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
          </Field>
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={internal}
              onChange={(e) => setInternal(e.target.checked)}
            />
            <span className="text-xs">Internal (no external traffic)</span>
          </label>
        </Section>

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="actions">
          <div className="flex gap-2">
            <Button
              variant="ghost"
              type="button"
              onClick={() => nav(`/h/${hid}/networks`)}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !name.trim()}
              disallowReason={denyReason}
              className="flex-1"
            >
              {pending ? "…" : "Create"}
            </Button>
          </div>
        </Section>
      </form>
    </Page>
  );
}
