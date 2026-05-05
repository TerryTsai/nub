interface Option<T extends string> {
  value: T;
  label: string;
  count?: number;
}

/** Horizontal pill tabs sitting in the filter row beneath the breadcrumb.
 * Active option uses the sky-soft accent fill, others are plain. Mirrors
 * foundry's Inbox / Status / Agent / Created / Tag bar. */
export function Filters<T extends string>({
  options,
  value,
  onChange,
}: {
  options: Option<T>[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex gap-1 overflow-x-auto no-scrollbar -mx-5 px-5">
      {options.map((o) => {
        const active = o.value === value;
        const cls = active
          ? "shrink-0 px-2.5 py-1.5 rounded-full text-xs border border-[var(--accent-border)] bg-[var(--accent-soft)] text-[var(--accent)] active:opacity-80 transition-opacity"
          : "shrink-0 px-2.5 py-1.5 rounded-full text-xs border border-transparent text-[var(--text-tertiary)] hover:text-[var(--text-secondary)] active:opacity-80 transition-opacity";
        return (
          <button key={o.value} type="button" onClick={() => onChange(o.value)} className={cls}>
            {o.label}
            {o.count !== undefined && (
              <span className="ml-1.5 opacity-60">{o.count}</span>
            )}
          </button>
        );
      })}
    </div>
  );
}
