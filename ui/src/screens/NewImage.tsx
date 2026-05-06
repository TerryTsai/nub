import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import { call, streamOp, unwrap, type Host } from "@/api/client";
import type { DockerfileContent, DockerfileSummary } from "@/api/types";
import { useHosts } from "@/state/hosts";
import { invalidate, useQuery } from "@/state/cache";
import { parseArgs, type DockerfileArg } from "@/state/dockerfileArgs";
import { BuildLog } from "@/components/BuildLog";
import { Button } from "@/components/Button";
import { Collapsible } from "@/components/Collapsible";
import { Combobox } from "@/components/Combobox";
import { EditCell } from "@/components/EditCell";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, EMPTY_PULL, type PullState } from "@/components/PullProgress";
import { Row } from "@/components/Row";
import { scrollFocusedIntoView } from "@/lib/scrollIntoViewOnFocus";

/** "still-building" = WS dropped after we'd already received some progress.
 * Engine keeps building; we wait for the tag to appear in list_images. */
type Phase = "idle" | "running" | "still-building" | "done";

interface BuildState {
  stream: string;
  imageId: string | null;
}

const EMPTY_BUILD: BuildState = { stream: "", imageId: null };

export function NewImage() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  // Single field for both modes: the image:tag we either pull or build into.
  const [imageRef, setImageRef] = useState("");
  // Mode is implicit: empty `dockerfileName` → pull; set → build.
  const [dockerfileName, setDockerfileName] = useState("");
  const [dockerfileContent, setDockerfileContent] = useState("");
  const [args, setArgs] = useState<DockerfileArg[]>([]);
  const [argValues, setArgValues] = useState<Record<string, string>>({});

  const [phase, setPhase] = useState<Phase>("idle");
  const [pull, setPull] = useState<PullState>(EMPTY_PULL);
  const [build, setBuild] = useState<BuildState>(EMPTY_BUILD);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const dfKey = host ? `${host.url}:list_dockerfiles` : null;
  const { data: dockerfiles } = useQuery<DockerfileSummary[]>(dfKey, async () => {
    const r = unwrap(await call(host!, { op: "list_dockerfiles" }), "dockerfiles");
    return r.data;
  });

  const isBuild = dockerfileName.trim() !== "";

  // When a dockerfile is picked, fetch its content (used to parse ARGs
  // and to send as the build context — the build op no longer reads from
  // the server's dockerfiles directory).
  useEffect(() => {
    if (!host || !dockerfileName) {
      setDockerfileContent("");
      setArgs([]);
      setArgValues({});
      return;
    }
    let cancelled = false;
    (async () => {
      try {
        const r = unwrap(
          await call(host, { op: "get_dockerfile", name: dockerfileName }),
          "dockerfile",
        );
        if (cancelled) return;
        const content = (r.data as DockerfileContent).content;
        setDockerfileContent(content);
        const parsed = parseArgs(content);
        setArgs(parsed);
        setArgValues(Object.fromEntries(parsed.map((a) => [a.name, a.default ?? ""])));
      } catch (e) {
        if (!cancelled) setError(`load dockerfile: ${(e as Error).message}`);
      }
    })();
    return () => { cancelled = true; };
  }, [host?.url, host?.token, dockerfileName]);

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!host) return;
    const ref = imageRef.trim();
    if (!ref) return;
    setError(null);
    abortRef.current = new AbortController();
    if (!isBuild) {
      setPhase("running");
      setPull({ layers: {}, lastStatus: "starting pull…" });
      try {
        await streamOp(
          host,
          { op: "pull_image", reference: ref },
          (chunk) => setPull((prev) => reducePull(prev, chunk)),
          abortRef.current.signal,
        );
        invalidate(`${host.url}:list_images`);
        setPhase("done");
      } catch (err) {
        setError(`pull image: ${(err as Error).message}`);
        setPhase("idle");
      }
    } else {
      if (!dockerfileContent) {
        setError("dockerfile content not loaded yet");
        return;
      }
      setPhase("running");
      setBuild(EMPTY_BUILD);
      let receivedAny = false;
      try {
        await streamOp(
          host,
          { op: "build_image", dockerfile_content: dockerfileContent, tag: ref, build_args: argValues },
          (chunk) => {
            if (chunk.type !== "build_progress") return;
            receivedAny = true;
            setBuild((prev) => ({
              stream: prev.stream + chunk.stream,
              imageId: chunk.image_id ?? prev.imageId,
            }));
          },
          abortRef.current.signal,
        );
        invalidate(`${host.url}:list_images`);
        setPhase("done");
      } catch (err) {
        // If the WS dropped after the build had started, the engine keeps
        // building. Switch to a poll-for-the-tag state instead of blowing
        // up — the user just needs to wait for the image to land.
        if (receivedAny && !abortRef.current?.signal.aborted) {
          setError(null);
          setPhase("still-building");
          watchForTag(host, ref).then((found) => {
            if (found) {
              invalidate(`${host.url}:list_images`);
              setPhase("done");
            } else {
              setError("build still running but the image hasn't landed in 2 minutes — check the host directly");
              setPhase("idle");
            }
          });
        } else {
          setError(`build image: ${(err as Error).message}`);
          setPhase("idle");
        }
      }
    }
  }

  /** Poll list_images every 5s for up to 2 minutes, looking for `tag`.
   * Resolves true when found, false if the budget is exhausted. */
  async function watchForTag(h: Host, tag: string): Promise<boolean> {
    const deadline = Date.now() + 120_000;
    while (Date.now() < deadline) {
      await new Promise((r) => setTimeout(r, 5000));
      try {
        const r = unwrap(await call(h, { op: "list_images" }), "images");
        if (r.data.some((img) => img.repo_tag === tag)) return true;
      } catch {
        // ignore transient errors during polling
      }
    }
    return false;
  }

  function onCancel() {
    abortRef.current?.abort();
    setPhase("idle");
    setPull(EMPTY_PULL);
    setBuild(EMPTY_BUILD);
  }

  function reset() {
    setPhase("idle");
    setPull(EMPTY_PULL);
    setBuild(EMPTY_BUILD);
    setError(null);
    setImageRef("");
    // Keep dockerfileName/args so the user can iterate quickly.
  }

  const sectionCrumbs = useHostSectionCrumbs(hid ?? "", saved?.label ?? "?", "images");

  if (!saved) return <Page><p>Unknown host.</p></Page>;

  const crumbs: Crumb[] = [
    ...sectionCrumbs,
    { kind: "link", label: "new image" },
  ];

  const dfOptions = (dockerfiles ?? []).map((d) => ({ value: d.name }));
  const running = phase === "running" || phase === "still-building";
  const submitDisabled = running || !imageRef.trim();

  const submitLabel = phase === "running"
    ? isBuild ? "Building…" : "Pulling…"
    : phase === "still-building"
    ? "Waiting…"
    : isBuild ? "Build" : "Pull";

  return (
    <Page crumbs={crumbs}>
      {error && <p className="text-[var(--error)] text-xs">{error}</p>}

      <form onSubmit={onSubmit} className="contents" {...scrollFocusedIntoView()}>
        <Collapsible label="Image" defaultOpen>
          <Row
            label="Name"
            right={
              <EditCell
                mono
                value={imageRef}
                placeholder="image:tag"
                onChange={(e) => setImageRef(e.target.value)}
              />
            }
          />
          <Row
            label="From"
            right={
              <Combobox
                cell
                mono
                dim={!isBuild}
                value={dockerfileName}
                onChange={setDockerfileName}
                options={[
                  { value: "", label: "registry" },
                  ...dfOptions,
                ]}
              />
            }
          />
        </Collapsible>

        {isBuild && args.length > 0 && (
          <Collapsible label="build args" count={args.length} defaultOpen>
            {args.map((a) => (
              <Row
                key={a.name}
                label={a.name}
                right={
                  <EditCell
                    mono
                    value={argValues[a.name] ?? ""}
                    placeholder={a.default ?? "value"}
                    onChange={(e) => setArgValues({ ...argValues, [a.name]: e.target.value })}
                    disabled={running}
                  />
                }
              />
            ))}
          </Collapsible>
        )}

        {phase !== "idle" && (
          <Collapsible label="progress" defaultOpen>
            {!isBuild ? <PullProgress pull={pull} /> : <BuildLog stream={build.stream} imageId={build.imageId} />}
            {phase === "still-building" && (
              <p className="text-xs text-[var(--text-secondary)] pt-2">
                connection dropped, but the build is still running on the host —
                waiting for <span className="mono">{imageRef}</span> to appear in the image list…
              </p>
            )}
          </Collapsible>
        )}

        <Collapsible label={isBuild ? "build" : "pull"} defaultOpen>
          {phase === "done" ? (
            <div className="flex gap-2">
              <Button variant="ghost" onClick={reset} className="flex-1">
                {isBuild ? "Build another" : "Pull another"}
              </Button>
              <Button onClick={() => nav(`/h/${hid}/images`)} className="flex-1">
                Done
              </Button>
            </div>
          ) : (
            <div className="flex gap-2">
              <Button
                variant="ghost"
                onClick={() => (phase === "running" ? onCancel() : nav(`/h/${hid}/images`))}
                className="flex-1"
              >
                {phase === "running" ? "Cancel" : phase === "still-building" ? "Leave" : "Back"}
              </Button>
              <Button type="submit" disabled={submitDisabled} className="flex-1">
                {submitLabel}
              </Button>
            </div>
          )}
        </Collapsible>
      </form>
    </Page>
  );
}
