/**
 * 服务等级选择器 - Mini/Pro/Max 三档选择
 *
 * 提供类似 v0 的简洁模式选择体验
 */

import React from "react";
import { Zap, Sparkles, Crown } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ServiceTier } from "@/lib/api/orchestrator";

interface TierOption {
  id: ServiceTier;
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
  bgColor: string;
  borderColor: string;
}

const tierOptions: TierOption[] = [
  {
    id: "mini",
    label: "Mini",
    description: "快速响应",
    icon: <Zap className="h-4 w-4" />,
    color: "text-green-600 dark:text-green-400",
    bgColor: "bg-green-50 dark:bg-green-950",
    borderColor: "border-green-200 dark:border-green-800",
  },
  {
    id: "pro",
    label: "Pro",
    description: "均衡性能",
    icon: <Sparkles className="h-4 w-4" />,
    color: "text-blue-600 dark:text-blue-400",
    bgColor: "bg-blue-50 dark:bg-blue-950",
    borderColor: "border-blue-200 dark:border-blue-800",
  },
  {
    id: "max",
    label: "Max",
    description: "最强能力",
    icon: <Crown className="h-4 w-4" />,
    color: "text-purple-600 dark:text-purple-400",
    bgColor: "bg-purple-50 dark:bg-purple-950",
    borderColor: "border-purple-200 dark:border-purple-800",
  },
];

interface TierSelectorProps {
  /** 当前选中的等级 */
  value: ServiceTier;
  /** 等级变化回调 */
  onChange: (tier: ServiceTier) => void;
  /** 是否禁用 */
  disabled?: boolean;
  /** 各等级的模型数量 */
  modelCounts?: {
    mini: number;
    pro: number;
    max: number;
  };
  /** 紧凑模式 */
  compact?: boolean;
  /** 自定义类名 */
  className?: string;
}

export function TierSelector({
  value,
  onChange,
  disabled = false,
  modelCounts,
  compact = false,
  className,
}: TierSelectorProps) {
  return (
    <div className={cn("flex gap-2", compact ? "gap-1" : "gap-2", className)}>
      {tierOptions.map((option) => {
        const isSelected = value === option.id;
        const count = modelCounts?.[option.id];
        const hasModels = count === undefined || count > 0;

        return (
          <button
            key={option.id}
            type="button"
            onClick={() => onChange(option.id)}
            disabled={disabled || !hasModels}
            className={cn(
              "flex-1 rounded-lg border px-3 py-2 transition-all",
              "focus:outline-none focus:ring-2 focus:ring-offset-2",
              compact ? "px-2 py-1.5" : "px-3 py-2",
              isSelected
                ? cn(
                    option.borderColor,
                    option.bgColor,
                    option.color,
                    "ring-2 ring-offset-1",
                    option.id === "mini" && "ring-green-500/50",
                    option.id === "pro" && "ring-blue-500/50",
                    option.id === "max" && "ring-purple-500/50",
                  )
                : cn(
                    "border-border hover:bg-muted",
                    !hasModels && "opacity-50 cursor-not-allowed",
                  ),
            )}
          >
            <div className="flex items-center justify-center gap-1.5">
              <span
                className={cn(
                  isSelected ? option.color : "text-muted-foreground",
                )}
              >
                {option.icon}
              </span>
              <span
                className={cn(
                  "font-medium",
                  compact ? "text-xs" : "text-sm",
                  isSelected ? option.color : "text-foreground",
                )}
              >
                {option.label}
              </span>
              {count !== undefined && (
                <span
                  className={cn(
                    "text-xs",
                    isSelected ? option.color : "text-muted-foreground",
                  )}
                >
                  ({count})
                </span>
              )}
            </div>
            {!compact && (
              <p
                className={cn(
                  "text-xs mt-0.5",
                  isSelected ? option.color : "text-muted-foreground",
                )}
              >
                {option.description}
              </p>
            )}
          </button>
        );
      })}
    </div>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export { tierOptions };
export type { TierOption };
