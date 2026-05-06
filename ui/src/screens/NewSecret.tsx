import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, unwrap, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { EditCell } from "@/components/EditCell";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { Row } from "@/components/Row";
import { Spinner } from "@/components/Spinner";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

export function NewSecret() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const [name, setName] = useState("");
  const [value, setValue] = useState("");
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    setPending(true);
    setError(null);
    try {
      unwrap(await call(host, { op: "put_secret", name: name.trim(), value }), "ok");
      invalidate(`${host.url}:list_secrets`);
      nav(`/h/${hid}/secrets`, { replace: true });
    } catch (e) {
      setError(`create secret: ${(e as Error).message}`);
    } finally {
      setPending(false);
    }
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "secrets");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [...sectionCrumbs, { kind: "link", label: "new secret" }];

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Collapsible label="Secret" defaultOpen>
          <Row
            label="Name"
            right={
              <EditCell
                mono
                value={name}
                placeholder="new secret"
                onChange={(e) => setName(e.target.value)}
              />
            }
          />
        </Collapsible>

        <Collapsible label="value" defaultOpen>
          <textarea
            className="input-code"
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
        </Collapsible>

        <Collapsible label="create" defaultOpen>
          <div className="flex gap-2">
            <Button
              variant="ghost"
              onClick={() => nav(`/h/${hid}/secrets`)}
              className="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={pending || !name.trim() || !value}
              className="flex-1"
            >
              {pending ? (<><Spinner /> Creating…</>) : "Create"}
            </Button>
          </div>
        </Collapsible>
      </form>
    </Page>
  );
}
