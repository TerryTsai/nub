/** Shimmer placeholder used while data is in-flight. Matches the shape of
 * the row/section it's standing in for so the page doesn't jump when the
 * real content lands.
 *
 * Use `<SkeletonRows count={N} />` for list pages — produces a vertical
 * stack of placeholder rows whose height matches `<ListRow>`. Use the bare
 * `<Skeleton>` for one-off shapes (e.g. a section's worth of values). */
export function Skeleton({ className = "" }: { className?: string }) {
  return (
    <div
      className={`bg-[var(--bg-elevated)] border border-[var(--border-subtle)] rounded-[var(--radius-sm)] animate-pulse ${className}`}
      aria-hidden="true"
    />
  );
}

export function SkeletonRows({ count = 4 }: { count?: number }) {
  return (
    <div className="flex flex-col" aria-busy="true" aria-live="polite">
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className="border-b border-[var(--border-subtle)] last:border-b-0 py-3.5 flex items-center gap-3"
        >
          <div className="flex-1 min-w-0 flex flex-col gap-1.5">
            <Skeleton className="h-3 w-1/2" />
            <Skeleton className="h-2.5 w-3/4 opacity-60" />
          </div>
        </div>
      ))}
    </div>
  );
}
