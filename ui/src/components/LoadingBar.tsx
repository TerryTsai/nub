/** Tiny indeterminate progress strip — 2px tall, full-width, sky bar
 * sliding L→R on a transparent track. Render at the top of the page body
 * while async data is in flight. The body stays blank below so the user
 * sees nothing pop in or jump when the response lands.
 *
 * Bleeds past the body's px-5/pt-3 padding via negative margins so the
 * strip sits flush against the breadcrumb header above. */
export function LoadingBar() {
  return (
    <div
      className="loading-bar h-0.5 -mx-5 -mt-3 overflow-hidden"
      role="status"
      aria-label="loading"
    >
      <span className="block h-full w-1/3 bg-[var(--accent)]" />
    </div>
  );
}
