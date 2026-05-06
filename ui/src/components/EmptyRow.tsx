/** Single-line em-dash placeholder for a list-shaped Collapsible whose
 * body is empty. Mirrors how Row renders a missing value, so an empty
 * section reads the same as an absent field elsewhere on the page. */
export function EmptyRow() {
  return (
    <span className="text-xs text-[var(--text-tertiary)]">—</span>
  );
}
