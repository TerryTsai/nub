import type { ButtonHTMLAttributes, ReactNode } from "react";

type Variant = "primary" | "ghost" | "destructive";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  children: ReactNode;
  /** When set, the button renders disabled and the title gives the reason. */
  disallowReason?: string;
}

export function Button({
  variant = "primary",
  disallowReason,
  className = "",
  children,
  disabled,
  title,
  ...rest
}: Props) {
  const isDisabled = disabled || !!disallowReason;
  return (
    <button
      type="button"
      className={`btn btn-${variant} ${className}`}
      disabled={isDisabled}
      title={disallowReason ?? title}
      aria-disabled={isDisabled}
      {...rest}
    >
      {children}
    </button>
  );
}
