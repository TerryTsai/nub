import { useEffect, useRef, useState } from "react";
import { useNavigate, useParams, useSearchParams } from "react-router-dom";
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
import { Heading } from "@/components/Heading";
import { useHostSectionCrumbs } from "@/components/HostCrumbs";
import { Page, type Crumb } from "@/components/Page";
import { PullProgress, reducePull, EMPTY_PULL, type PullState } from "@/components/PullProgress";
import { Row } from "@/components/Row";
import { Section } from "@/components/Section";

type Source = "pull" | "build";
/** "still-building" = WS dropped after we'd already received some progress.
 * Engine keeps building; we wait for the tag to appear in list_images. */
type Phase = "idle" | "running" | "still-building" | "done";

interface BuildState {
  stream: string;
  imageId: string | null;
}

const EMPTY_BUILD: BuildState = { stream: "", imageId: null };

const SOURCE_OPTIONS = [
  { value: "pull", label: "pull" },
  { value: "build", label: "build" },
];

export function NewImage() {
  const { hid } = useParams<{ hid: string }>();
  const nav = useNavigate();
  const [params, setParams] = useSearchParams();
  const { hosts } = useHosts();
  const saved = hosts.find((h) => h.hid === hid);
  const host: Host | undefined = saved && { url: saved.url, token: saved.token };

  const source: Source = params.get("source") === "build" ? "build" : "pull";
  const setSource = (s: Source) => setParams({ source: s }, { replace: true });

  // Single field for both modes: the image:tag we either pull or build into.
  const [imageRef, setImageRef] = useState("");

  // Build-only state
  const [dockerfileName, setDockerfileName] = useState("");
  const [args, setArgs] = useState<DockerfileArg[]>([]);
  const [argValues, setArgValues] = useState<Record<string, string>>({});

  // Shared
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

  // When a dockerfile is picked, fetch its content and surface ARGs.
  useEffect(() => {
    if (!host || !dockerfileName) {
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
        const parsed = parseArgs((r.data as DockerfileContent).content);
        setArgs(parsed);
        setArgValues(Object.fromEntries(parsed.map((a) => [a.name, a.default ?? ""])));
      } catch (e) {
        if (!cancelled) setError((e as Error).message);
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
    if (source === "pull") {
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
        setError((err as Error).message);
        setPhase("idle");
      }
    } else {
      const df = dockerfileName.trim();
      if (!df) return;
      setPhase("running");
      setBuild(EMPTY_BUILD);
      let receivedAny = false;
      try {
        await streamOp(
          host,
          { op: "build_image", dockerfile: df, tag: ref, build_args: argValues },
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
          setError((err as Error).message);
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
  const submitDisabled =
    running ||
    !imageRef.trim() ||
    (source === "build" && !dockerfileName.trim());

  const submitLabel = phase === "running"
    ? source === "pull" ? "Pulling…" : "Building…"
    : phase === "still-building"
    ? "Waiting…"
    : source === "pull" ? "Pull" : "Build";

  return (
    <Page crumbs={crumbs}>
      <Heading
        category="Image"
        editable={{
          value: imageRef,
          onChange: setImageRef,
          placeholder: "image:tag",
        }}
      />

      <form onSubmit={onSubmit} className="contents">
        <Section>
          <Row
            label="Source"
            right={
              <Combobox
                cell
                dim={source === "pull"}
                value={source}
                onChange={(v) => setSource(v as Source)}
                options={SOURCE_OPTIONS}
              />
            }
          />
          {source === "build" && (
            <Row
              label="Dockerfile"
              right={
                <Combobox
                  cell
                  mono
                  value={dockerfileName}
                  onChange={setDockerfileName}
                  placeholder={dfOptions.length === 0 ? "no dockerfiles yet" : "pick…"}
                  options={dfOptions}
                />
              }
            />
          )}
        </Section>

        {source === "build" && args.length > 0 && (
          <Collapsible label="build args" count={args.length} defaultOpen>
            {args.map((a) => (
              <Row
                key={a.name}
                label={a.name}
                right={
                  <EditCell
                    mono
                    placeholder={a.default ?? ""}
                    value={argValues[a.name] ?? ""}
                    onChange={(e) => setArgValues({ ...argValues, [a.name]: e.target.value })}
                    disabled={running}
                  />
                }
              />
            ))}
          </Collapsible>
        )}

        {phase !== "idle" && (
          <Section label="progress">
            {source === "pull" ? <PullProgress pull={pull} /> : <BuildLog stream={build.stream} imageId={build.imageId} />}
            {phase === "still-building" && (
              <p className="text-xs text-[var(--text-secondary)] pt-2">
                connection dropped, but the build is still running on the host —
                waiting for <span className="mono">{imageRef}</span> to appear in the image list…
              </p>
            )}
          </Section>
        )}

        {error && <p className="text-[var(--error)] text-xs">{error}</p>}

        <Section label={source === "pull" ? "pull" : "build"}>
          {phase === "done" ? (
            <div className="flex gap-2">
              <Button variant="ghost" onClick={reset} className="flex-1">
                {source === "pull" ? "Pull another" : "Build another"}
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
        </Section>
      </form>
    </Page>
  );
}
