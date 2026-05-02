/** Tiny SVG line chart for ~60 samples of a time series. Fills the
 * container width; fixed height (28px). The SVG uses preserveAspectRatio
 * "none" so the line stretches edge-to-edge regardless of width. */
export function Sparkline({
  values,
  max,
  capacity,
  className = "",
}: {
  values: number[];
  /** Top of the y-axis. Defaults to `Math.max(...values, 1)`. Override to
   * keep a stable scale (e.g. 100 for CPU%, mem_limit for memory). */
  max?: number;
  /** Buffer size — leaves trailing empty space until the buffer fills,
   * so a fresh sparkline grows in from the left rather than stretching
   * one sample across the whole width. */
  capacity?: number;
  className?: string;
}) {
  const width = 100;
  const height = 28;
  const cap = capacity ?? values.length;
  const ymax = Math.max(max ?? Math.max(...values, 1), 0.0001);

  const pts = values.map((v, i) => {
    const x = cap > 1 ? (i / (cap - 1)) * width : 0;
    const y = height - Math.max(0, Math.min(1, v / ymax)) * height;
    return `${x.toFixed(2)},${y.toFixed(2)}`;
  });

  return (
    <svg
      viewBox={`0 0 ${width} ${height}`}
      preserveAspectRatio="none"
      className={`w-full h-7 ${className}`}
      aria-hidden="true"
    >
      {pts.length > 1 && (
        <>
          <polyline
            points={`0,${height} ${pts.join(" ")} ${pts[pts.length - 1].split(",")[0]},${height}`}
            fill="var(--accent-soft)"
            stroke="none"
          />
          <polyline
            points={pts.join(" ")}
            fill="none"
            stroke="var(--accent)"
            strokeWidth={1}
            vectorEffect="non-scaling-stroke"
          />
        </>
      )}
    </svg>
  );
}
