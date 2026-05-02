/** Small dot + lowercase status label. Used in list rows and page headers
 * to show container/op state at a glance. */
export function StatusBadge({ status }: { status: string }) {
  return (
    <span className="flex items-center gap-1.5 shrink-0">
      <span className={`dot dot-${status}`} aria-label={status} />
      <span className="text-[11px] text-[var(--text-secondary)] capitalize">{status}</span>
    </span>
  );
}
