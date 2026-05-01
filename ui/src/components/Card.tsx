import type { ReactNode, HTMLAttributes } from "react";

export function Card({
  children,
  className = "",
  ...rest
}: { children: ReactNode } & HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={`glass rounded-[var(--radius-lg)] p-5 flex flex-col gap-3 ${className}`}
      {...rest}
    >
      {children}
    </div>
  );
}

/** Key/value row used inside Cards. */
export function Row({
  label,
  value,
  mono,
  right,
}: {
  label: string;
  value?: string;
  mono?: boolean;
  right?: ReactNode;
}) {
  return (
    <div className="flex justify-between items-baseline gap-3">
      <span className="text-sm text-[var(--text-secondary)]">{label}</span>
      {right ? right : (
        <span className={`text-sm text-right break-all ${mono ? "mono" : ""}`}>{value}</span>
      )}
    </div>
  );
}
