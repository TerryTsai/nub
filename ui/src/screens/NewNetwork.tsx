import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

const NAME_RE = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;
const NAME_HINT = "letters, digits, dot, underscore, dash; must start with letter/digit";

export function NewNetwork() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [name, setName] = useState("");
  const [internal, setInternal] = useState(false);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [nameError, setNameError] = useState<string | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const trimmed = name.trim();
    setNameError(null);
    if (!NAME_RE.test(trimmed)) {
      setNameError("name must start with a letter or digit; letters/digits/dot/underscore/dash only");
      return;
    }
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "create_network", name: trimmed, internal }), "ok");
      invalidate(`${host.url}:list_networks`);
      nav(`/h/${hid}/networks`, { replace: true });
    } catch (e) {
      setError(`create network: ${(e as Error).message}`);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "networks");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new network" }];

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Network"
        editable={{
          value: name,
          onChange: setName,
          placeholder: "new network",
        }}
      />
      {nameError ? (
        <p className="text-[11px] text-[var(--error)]">{nameError}</p>
      ) : (
        <p className="text-[11px] text-[var(--text-tertiary)]">{NAME_HINT}</p>
      )}

      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Section>
          <Row
            label="Internal"
            right={
              <label className="flex items-center gap-2 cursor-pointer text-xs text-[var(--text-secondary)]">
                <input
                  type="checkbox"
                  checked={internal}
                  onChange={(e) => setInternal(e.target.checked)}
                />
                no external traffic
              </label>
            }
          />
        </Section>

        <Section label="create">
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
              className="flex-1"
            >
              {pending ? (<><Spinner /> Creating…</>) : "Create"}
            </Button>
          </div>
        </Section>
      </form>
    </Page>
  );
}
