import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Section } from "@/components/Section";

const NAME_RE = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;

export function NewSecret() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [name, setName] = useState("");
  const [value, setValue] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const denyReason =
    session.session && !session.session.can("secrets:put")
      ? "your token doesn't allow secrets:put"
      : undefined;

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const trimmed = name.trim();
    if (!NAME_RE.test(trimmed) || trimmed.length > 128) {
      setError("name must start with a letter or digit; letters/digits/dot/underscore/dash only");
      return;
    }
    if (value.length === 0) {
      setError("value can't be empty");
      return;
    }
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "put_secret", name: trimmed, value }), "ok");
      invalidate(`${host.url}:list_secrets`);
      nav(`/h/${hid}/secrets`, { replace: true });
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "secrets");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new secret" }];

  return (
    <Page crumbs={crumbs}>
      <Heading category="Secret" title="new secret" />

      {denyReason && <p className="text-[var(--warn)] text-xs">{denyReason}</p>}

      <form onSubmit={onSubmit} className="contents">
        <Section label="secret">
          <Field label="Name">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="db_password"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
          </Field>
          <Field label="Value">
            <textarea
              className="input mono"
              spellCheck={false}
              autoCapitalize="off"
              autoCorrect="off"
              rows={4}
              placeholder="value…"
              value={value}
              onChange={(e) => setValue(e.target.value)}
              required
              style={{ minHeight: "96px" }}
            />
          </Field>
        </Section>

        <Collapsible>
          <p className="text-xs text-[var(--text-tertiary)]">
            Encrypted at rest with the host's age key. Values are never read back over
            the network — use <code className="mono">nub secret get NAME</code> on the host.
          </p>
        </Collapsible>

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="actions">
          <div className="flex gap-2">
            <Button
              variant="ghost"
              type="button"
              onClick={() => nav(`/h/${hid}/secrets`)}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !name.trim() || !value}
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
