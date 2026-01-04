/**
 * 模型列表组件 - 显示可用模型
 */

import { Check, AlertCircle, Loader2 } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AvailableModel } from "@/lib/api/orchestrator";

interface ModelListProps {
  /** 模型列表 */
  models: AvailableModel[];
  /** 选中的模型 ID */
  selectedModelId?: string;
  /** 选择模型回调 */
  onSelectModel?: (model: AvailableModel) => void;
  /** 是否加载中 */
  loading?: boolean;
  /** 错误信息 */
  error?: string | null;
  /** 自定义类名 */
  className?: string;
}

export function ModelList({
  models,
  selectedModelId,
  onSelectModel,
  loading = false,
  error = null,
  className,
}: ModelListProps) {
  if (loading) {
    return (
      <div className={cn("flex items-center justify-center py-8", className)}>
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载模型列表...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={cn(
          "flex items-center justify-center py-8 text-destructive",
          className,
        )}
      >
        <AlertCircle className="h-5 w-5 mr-2" />
        <span>{error}</span>
      </div>
    );
  }

  if (models.length === 0) {
    return (
      <div className={cn("text-center py-8 text-muted-foreground", className)}>
        <p>暂无可用模型</p>
        <p className="text-sm mt-1">请先添加凭证</p>
      </div>
    );
  }

  return (
    <div className={cn("space-y-2", className)}>
      {models.map((model) => {
        const isSelected = model.model_id === selectedModelId;

        return (
          <button
            key={`${model.model_id}-${model.credential_id}`}
            type="button"
            onClick={() => onSelectModel?.(model)}
            className={cn(
              "w-full flex items-center justify-between p-3 rounded-lg border transition-colors",
              "hover:bg-muted/50",
              isSelected ? "border-primary bg-primary/5" : "border-border",
              !model.is_healthy && "opacity-60",
            )}
          >
            <div className="flex items-center gap-3">
              {/* 选中指示器 */}
              <div
                className={cn(
                  "w-4 h-4 rounded-full border-2 flex items-center justify-center",
                  isSelected
                    ? "border-primary bg-primary"
                    : "border-muted-foreground",
                )}
              >
                {isSelected && (
                  <Check className="h-3 w-3 text-primary-foreground" />
                )}
              </div>

              {/* 模型信息 */}
              <div className="text-left">
                <div className="font-medium text-sm">{model.display_name}</div>
                <div className="text-xs text-muted-foreground">
                  {model.provider_type}
                  {model.context_length &&
                    ` · ${formatContextLength(model.context_length)}`}
                </div>
              </div>
            </div>

            {/* 状态和能力标签 */}
            <div className="flex items-center gap-2">
              {model.supports_vision && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-blue-100 dark:bg-blue-900 text-blue-700 dark:text-blue-300">
                  视觉
                </span>
              )}
              {model.supports_tools && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-green-100 dark:bg-green-900 text-green-700 dark:text-green-300">
                  工具
                </span>
              )}
              {!model.is_healthy && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-red-100 dark:bg-red-900 text-red-700 dark:text-red-300">
                  不健康
                </span>
              )}
              {model.current_load !== undefined && (
                <LoadIndicator load={model.current_load} />
              )}
            </div>
          </button>
        );
      })}
    </div>
  );
}

/** 负载指示器 */
function LoadIndicator({ load }: { load: number }) {
  const color =
    load < 30 ? "bg-green-500" : load < 70 ? "bg-yellow-500" : "bg-red-500";

  return (
    <div className="flex items-center gap-1">
      <div className="w-8 h-1.5 bg-muted rounded-full overflow-hidden">
        <div
          className={cn("h-full rounded-full", color)}
          style={{ width: `${load}%` }}
        />
      </div>
      <span className="text-xs text-muted-foreground">{load}%</span>
    </div>
  );
}

/** 格式化上下文长度 */
function formatContextLength(length: number): string {
  if (length >= 1000000) {
    return `${(length / 1000000).toFixed(1)}M`;
  }
  if (length >= 1000) {
    return `${(length / 1000).toFixed(0)}K`;
  }
  return String(length);
}
