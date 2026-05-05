import type { ReactNode } from "react";

interface Props {
  /** Small caps category line above the title, e.g. "CONTAINER", "SESSION". */
  category: string;
  /** Static page title. Ignored when `editable` is set. */
  title?: string;
  /** Right-aligned content on the title row — typically a `<StatusBadge />`. */
  right?: ReactNode;
  /** When set, the title is rendered as an inline-edit input instead of an
   * h1. Use on create/clone screens where the title IS the entity's name —
   * the form-as-detail pattern collapses the heading and the name field
   * into one. Font size is 16px so iOS doesn't auto-zoom on focus. */
  editable?: {
    value: string;
    onChange: (v: string) => void;
    placeholder: string;
  };
}

/** Detail-page heading: small-caps category line, then the title with an
 * optional badge inline at the right. Mirrors foundry's
 * TICKET / Simple site / ● Failed pattern. Sized to fit the body — the title
 * is medium-weight, not a display-font hero. */
export function Heading({ category, title, right, editable }: Props) {
  return (
    <header className="flex flex-col gap-1">
      <span className="text-[10px] font-semibold uppercase tracking-wider text-[var(--text-tertiary)]">
        {category}
      </span>
      <div className="flex items-center justify-between gap-3">
        {editable ? (
          <input
            type="text"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck={false}
            placeholder={editable.placeholder}
            value={editable.value}
            onChange={(e) => editable.onChange(e.target.value)}
            className="bg-transparent border-0 border-b border-dashed border-[var(--border-subtle)] hover:border-[var(--border-strong)] focus:border-[var(--accent)] focus:border-solid focus:outline-none text-[16px] font-semibold flex-1 min-w-0 transition-colors placeholder:text-[var(--text-tertiary)] placeholder:font-normal placeholder:italic"
          />
        ) : (
          <h1 className="text-base font-semibold truncate">{title}</h1>
        )}
        {right}
      </div>
    </header>
  );
}
