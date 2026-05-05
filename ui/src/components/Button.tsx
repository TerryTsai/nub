import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "ghost" | "destructive";
type Size = "default" | "sm";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  /** "sm" produces a slimmer pill — used in sub-nav rows where a normal
   * button would dominate the slim 40px header strip. */
  size?: Size;
  children: ReactNode;
}

export function Button({
  variant = "primary",
  size = "default",
  className = "",
  children,
  disabled,
  ...rest
}: Props) {
  const sizeCls = size === "sm" ? "btn-sm" : "";
  return (
    <button
      type="button"
      className={`btn btn-${variant} ${sizeCls} ${className}`}
      disabled={disabled}
      aria-disabled={disabled}
      {...rest}
    >
      {children}
    </button>
  );
}
