import clsx from "clsx";
import React from "react";

export function Button({
  children,
  className,
  variant = "primary",
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "ghost" | "outline" | "danger";
}) {
  return (
    <button
      className={clsx("btn", `btn-${variant}`, className)}
      {...props}
    >
      {children}
    </button>
  );
}

export function Card({
  children,
  className,
}: {
  children: React.ReactNode;
  className?: string;
}) {
  return <div className={clsx("card", className)}>{children}</div>;
}

export function Badge({
  children,
  tone = "neutral",
}: {
  children: React.ReactNode;
  tone?: "neutral" | "success" | "warning" | "danger" | "info";
}) {
  return <span className={clsx("badge", `badge-${tone}`)}>{children}</span>;
}

export function Pill({
  children,
  active,
  onClick,
}: {
  children: React.ReactNode;
  active?: boolean;
  onClick?: () => void;
}) {
  return (
    <button
      className={clsx("pill", active && "pill-active")}
      onClick={onClick}
      type="button"
    >
      {children}
    </button>
  );
}

export function SectionTitle({ children }: { children: React.ReactNode }) {
  return <h3 className="section-title">{children}</h3>;
}
