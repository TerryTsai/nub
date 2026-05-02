import { useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { streamOp, type Host } from "@/api/client";
import { useHosts } from "@/state/hosts";
import { useSession } from "@/state/session";
import { invalidate } from "@/state/cache";
import { Button } from "@/components/Button";
import { Field } from "@/components/Field";
import { Heading } from "@/components/Heading";
import { useHostCrumb } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, EMPTY_PULL, type PullState } from "@/components/PullProgress";
import { Section } from "@/components/Section";

type Phase = "idle" | "pulling" | "done";

export function PullImage() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };
  const session = useSession(host);

  const [reference, setReference] = useState("");
  const [phase, setPhase] = useState<Phase>("idle");
  const [pull, setPull] = useState<PullState>(EMPTY_PULL);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const denyReason =
    session.session && !session.session.can("pull_image")
      ? "your token doesn't allow pull_image"
      : undefined;

  async function onPull(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const ref = reference.trim();
    if (!ref) return;
    setPhase("pulling");
    setPull({ layers: {}, lastStatus: "starting pull…" });
    setError(null);
    abortRef.current = new AbortController();
    try {
      await streamOp(
        host,
        { op: "pull_image", reference: ref },
        (chunk) => setPull((prev) => reducePull(prev, chunk)),
        abortRef.current.signal,
      );
      invalidate(`${host.url}:list_images`);
      setPhase("done");
    } catch (e) {
      setError((e as Error).message);
      setPhase("idle");
    }
  }

  function onCancel() {
    abortRef.current?.abort();
    setPhase("idle");
    setPull(EMPTY_PULL);
  }

  const hostCrumb = useHostCrumb(hid ?? "", saved?.label ?? "?");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [hostCrumb, { kind: "link", label: "pull image" }];

  return (
    <Page crumbs={crumbs}>
      <Heading category="Images" title="Pull image" />

      {denyReason && <p className="text-[var(--warn)] text-xs">{denyReason}</p>}

      <form onSubmit={onPull} className="contents">
        <Section label="Reference">
          <Field label="Image" hint="e.g. nginx:alpine, postgres:16, ghcr.io/owner/repo:tag">
            <input
              className="input mono"
              type="text"
              autoCapitalize="off"
              autoCorrect="off"
              spellCheck={false}
              placeholder="image:tag"
              value={reference}
              onChange={(e) => setReference(e.target.value)}
              disabled={phase === "pulling"}
              required
            />
          </Field>
        </Section>

        {phase !== "idle" && (
          <Section label="Progress">
            <PullProgress pull={pull} />
          </Section>
        )}

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label="Actions">
          {phase === "done" ? (
            <div className="flex gap-2">
              <Button
                variant="ghost"
                onClick={() => {
                  setReference("");
                  setPhase("idle");
                  setPull(EMPTY_PULL);
                }}
                className="flex-1"
              >
                Pull another
              </Button>
              <Button onClick={() => nav(`/h/${hid}/images`)} className="flex-1">
                Done
              </Button>
            </div>
          ) : (
            <div className="flex gap-2">
              <Button
                variant="ghost"
                type="button"
                onClick={() => (phase === "pulling" ? onCancel() : nav(`/h/${hid}/images`))}
                className="flex-1"
              >
                {phase === "pulling" ? "Cancel" : "Back"}
              </Button>
              <Button
                type="submit"
                disabled={phase === "pulling" || !reference.trim()}
                disallowReason={denyReason}
                className="flex-1"
              >
                {phase === "pulling" ? "Pulling…" : "Pull"}
              </Button>
            </div>
          )}
        </Section>
      </form>
    </Page>
  );
}
