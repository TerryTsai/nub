import type { ReactNode } from "react";

interface Props {
  /** Small caps category line above the title, e.g. "CONTAINER", "SESSION". */
  category: string;
  /** Page title. Body weight — not a giant display heading. */
  title: string;
  /** Right-aligned content on the title row — typically a `<StatusBadge />`. */
  right?: ReactNode;
}

/** Detail-page heading: small-caps category line, then the title with an
 * optional badge inline at the right. Mirrors foundry's
 * TICKET / Simple site / ● Failed pattern. Sized to fit the body — the title
 * is medium-weight, not a display-font hero. */
export function Heading({ category, title, right }: Props) {
  return (
    <header className="flex flex-col gap-1">
      <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
        {category}
      </span>
      <div className="flex items-center justify-between gap-3">
        <h1 className="text-base font-semibold truncate">{title}</h1>
        {right}
      </div>
    </header>
  );
}
