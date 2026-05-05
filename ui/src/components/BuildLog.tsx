import { useEffect, useRef } from "react";

/** Streaming build output log. Lines are concatenated as the engine emits
 * them; auto-scrolls to the bottom while building. Mono, scrollable, fixed
 * max height. The image id (when known) renders separately. */
export function BuildLog({ stream, imageId }: { stream: string; imageId: string | null }) {
  const ref = useRef<HTMLPreElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [stream]);

  return (
    <div className="flex flex-col gap-2">
      <pre
        ref={ref}
        className="mono text-xs leading-5 max-h-64 overflow-auto bg-[var(--bg-elevated)] border border-[var(--border-subtle)] rounded-[var(--radius-md)] p-2 whitespace-pre-wrap break-all text-[var(--text-secondary)]"
      >
        {stream || "starting build…"}
      </pre>
      {imageId && (
        <div className="text-[11px] text-[var(--text-tertiary)]">
          image id <span className="mono text-[var(--id-color)]">{shortId(imageId)}</span>
        </div>
      )}
    </div>
  );
}

function shortId(id: string): string {
  const stripped = id.startsWith("sha256:") ? id.slice(7) : id;
  return stripped.slice(0, 12);
}
