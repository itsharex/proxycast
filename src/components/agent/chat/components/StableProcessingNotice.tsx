import React from "react";
import { ShieldCheck } from "lucide-react";
import { cn } from "@/lib/utils";
import {
  getStableProcessingDescription,
  shouldShowStableProcessingNotice,
  STABLE_PROCESSING_LABEL,
  type StableProcessingScope,
} from "../utils/stableProcessingExperience";

interface StableProcessingNoticeProps {
  providerType?: string | null;
  model?: string | null;
  scope?: StableProcessingScope;
  className?: string;
  testId?: string;
}

export const StableProcessingNotice: React.FC<StableProcessingNoticeProps> = ({
  providerType,
  model,
  scope = "request",
  className,
  testId = "stable-processing-notice",
}) => {
  if (!shouldShowStableProcessingNotice({ providerType, model })) {
    return null;
  }

  return (
    <div
      data-testid={testId}
      className={cn(
        "flex items-start gap-2 rounded-2xl border border-amber-200/80 bg-amber-50/90 px-3 py-2 text-[11px] leading-5 text-amber-900",
        className,
      )}
    >
      <div className="mt-0.5 flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-white/80 text-amber-700">
        <ShieldCheck className="h-3.5 w-3.5" />
      </div>
      <div className="min-w-0">
        <div className="font-semibold text-amber-800">
          {STABLE_PROCESSING_LABEL}
        </div>
        <div className="text-amber-700">
          {getStableProcessingDescription(scope)}
        </div>
      </div>
    </div>
  );
};
