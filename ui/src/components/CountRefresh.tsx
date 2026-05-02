/** Thin meta row with an item count on the left and a small "refresh" link
 * on the right. Sits at the top of a list — fills the spot a Section header
 * would have, but quieter (no big small-caps label). */
export function CountRefresh({
  label,
  onRefresh,
  refreshing,
}: {
  label: string;
  onRefresh: () => void;
  refreshing: boolean;
}) {
  return (
    <div className="flex justify-between items-center text-[11px] text-[var(--text-tertiary)]">
      <span>{label}</span>
      <button
        type="button"
        onClick={onRefresh}
        disabled={refreshing}
        className="hover:text-[var(--text-secondary)] disabled:opacity-40"
      >
        {refreshing ? "…" : "refresh"}
      </button>
    </div>
  );
}
