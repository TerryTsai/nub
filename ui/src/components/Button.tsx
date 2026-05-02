import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "ghost" | "destructive";
type Size = "default" | "sm";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  /** "sm" produces a slimmer pill — used in sub-nav rows where a normal
   * button would dominate the slim 40px header strip. */
  size?: Size;
  children: ReactNode;
  /** When set, the button renders disabled and the title gives the reason. */
  disallowReason?: string;
}

export function Button({
  variant = "primary",
  size = "default",
  disallowReason,
  className = "",
  children,
  disabled,
  title,
  ...rest
}: Props) {
  const isDisabled = disabled || !!disallowReason;
  const sizeCls = size === "sm" ? "btn-sm" : "";
  return (
    <button
      type="button"
      className={`btn btn-${variant} ${sizeCls} ${className}`}
      disabled={isDisabled}
      title={disallowReason ?? title}
      aria-disabled={isDisabled}
      {...rest}
    >
      {children}
    </button>
  );
}
