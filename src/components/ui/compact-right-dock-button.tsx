import React from "react";
import { cn } from "@/lib/utils";

type CompactRightDockTone = "default" | "active" | "error";

interface CompactRightDockButtonProps {
  icon: React.ReactNode;
  label: string;
  badgeLabel?: string;
  badgeTone?: CompactRightDockTone;
  ariaLabel: string;
  title?: string;
  testId?: string;
  onClick: () => void;
}

function resolveToneClassName(tone: CompactRightDockTone): string {
  switch (tone) {
    case "active":
      return "border-sky-200 bg-sky-50 text-sky-700 shadow-sky-950/10 hover:bg-sky-100 hover:text-sky-800";
    case "error":
      return "border-rose-200 bg-rose-50 text-rose-700 shadow-rose-950/10 hover:bg-rose-100 hover:text-rose-800";
    case "default":
    default:
      return "border-slate-200 bg-white text-slate-600 shadow-slate-950/10 hover:bg-slate-50 hover:text-slate-900";
  }
}

function resolveBadgeToneClassName(tone: CompactRightDockTone): string {
  switch (tone) {
    case "active":
      return "border-sky-200 bg-white/90 text-sky-700";
    case "error":
      return "border-rose-200 bg-white/90 text-rose-700";
    case "default":
    default:
      return "border-slate-200 bg-white/90 text-slate-600";
  }
}

export const CompactRightDockButton: React.FC<CompactRightDockButtonProps> = ({
  icon,
  label,
  badgeLabel,
  badgeTone = "default",
  ariaLabel,
  title,
  testId,
  onClick,
}) => {
  return (
    <button
      type="button"
      aria-label={ariaLabel}
      title={title}
      data-testid={testId}
      onClick={onClick}
      className={cn(
        "inline-flex h-10 items-center gap-2 rounded-full border px-3.5 text-sm font-medium shadow-lg transition-all hover:-translate-y-0.5",
        resolveToneClassName(badgeTone),
      )}
    >
      {icon}
      <span>{label}</span>
      {badgeLabel ? (
        <span
          className={cn(
            "rounded-full border px-2 py-0.5 text-[10px] font-semibold leading-none",
            resolveBadgeToneClassName(badgeTone),
          )}
        >
          {badgeLabel}
        </span>
      ) : null}
    </button>
  );
};
