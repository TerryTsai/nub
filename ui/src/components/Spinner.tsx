/** Tiny inline spinner. Used in pending button states ("Removing…") and
 * anywhere a discrete "in flight" affordance reads better than a bare
 * ellipsis. Inherits `currentColor` so it picks up the parent's text tone. */
export function Spinner({ className = "" }: { className?: string }) {
  return (
    <svg
      className={`inline-block animate-spin ${className}`}
      width="12"
      height="12"
      viewBox="0 0 12 12"
      aria-hidden="true"
    >
      <circle cx="6" cy="6" r="4.5" fill="none" stroke="currentColor" strokeOpacity="0.25" strokeWidth="1.5" />
      <path
        d="M6 1.5 A 4.5 4.5 0 0 1 10.5 6"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.5"
        strokeLinecap="round"
      />
    </svg>
  );
}
