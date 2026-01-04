/**
 * 统一模型选择器组件
 *
 * 整合简单模式（Mini/Pro/Max）和专家模式（直接选择模型）
 */

import { useState } from "react";
import { RefreshCw, Activity } from "lucide-react";
import { cn } from "@/lib/utils";
import { TierSelector } from "./TierSelector";
import { ModeToggle, type SelectionMode } from "./ModeToggle";
import { ModelList } from "./ModelList";
import {
  useOrchestrator,
  useModelSelection,
  type ServiceTier,
  type AvailableModel,
  type SelectionResult,
} from "@/lib/api/orchestrator";

interface ModelSelectorProps {
  /** 初始模式 */
  initialMode?: SelectionMode;
  /** 初始等级 */
  initialTier?: ServiceTier;
  /** 选择模型回调 */
  onSelect?: (result: SelectionResult) => void;
  /** 是否显示模式切换 */
  showModeToggle?: boolean;
  /** 是否显示统计信息 */
  showStats?: boolean;
  /** 紧凑模式 */
  compact?: boolean;
  /** 自定义类名 */
  className?: string;
}

export function ModelSelector({
  initialMode = "simple",
  initialTier = "pro",
  onSelect,
  showModeToggle = true,
  showStats = true,
  compact = false,
  className,
}: ModelSelectorProps) {
  const [mode, setMode] = useState<SelectionMode>(initialMode);
  const [selectedModel, setSelectedModel] = useState<AvailableModel | null>(
    null,
  );

  // 使用编排器状态
  const {
    initialized: _initialized,
    loading: orchestratorLoading,
    error: orchestratorError,
    poolStats,
    refreshStats,
  } = useOrchestrator();

  // 使用模型选择
  const {
    tier,
    setTier,
    models,
    loading: modelsLoading,
    error: modelsError,
    selectModel,
    refreshModels,
  } = useModelSelection(initialTier);

  // 简单模式下自动选择模型
  const handleTierChange = async (newTier: ServiceTier) => {
    setTier(newTier);

    if (mode === "simple") {
      try {
        const result = await selectModel({ tier: newTier });
        onSelect?.(result);
      } catch (err) {
        console.error("模型选择失败:", err);
      }
    }
  };

  // 专家模式下手动选择模型
  const handleModelSelect = async (model: AvailableModel) => {
    setSelectedModel(model);

    try {
      const result = await selectModel({
        tier,
        preferred_provider: model.provider_type,
      });
      onSelect?.(result);
    } catch (err) {
      console.error("模型选择失败:", err);
    }
  };

  // 刷新数据
  const handleRefresh = () => {
    refreshStats();
    refreshModels();
  };

  const loading = orchestratorLoading || modelsLoading;
  const error = orchestratorError || modelsError;

  return (
    <div className={cn("space-y-4", className)}>
      {/* 头部：模式切换和刷新 */}
      <div className="flex items-center justify-between">
        {showModeToggle && (
          <ModeToggle mode={mode} onModeChange={setMode} disabled={loading} />
        )}

        <div className="flex items-center gap-2">
          {/* 统计信息 */}
          {showStats && poolStats && (
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <Activity className="h-3 w-3" />
              <span>
                {poolStats.healthy_count}/{poolStats.total_count} 可用
              </span>
            </div>
          )}

          {/* 刷新按钮 */}
          <button
            type="button"
            onClick={handleRefresh}
            disabled={loading}
            className="p-1.5 rounded-md hover:bg-muted transition-colors"
            title="刷新"
          >
            <RefreshCw
              className={cn(
                "h-4 w-4 text-muted-foreground",
                loading && "animate-spin",
              )}
            />
          </button>
        </div>
      </div>

      {/* 等级选择器 */}
      <TierSelector
        value={tier}
        onChange={handleTierChange}
        disabled={loading}
        modelCounts={
          poolStats
            ? {
                mini: poolStats.mini_count,
                pro: poolStats.pro_count,
                max: poolStats.max_count,
              }
            : undefined
        }
        compact={compact}
      />

      {/* 专家模式：显示模型列表 */}
      {mode === "expert" && (
        <ModelList
          models={models}
          selectedModelId={selectedModel?.model_id}
          onSelectModel={handleModelSelect}
          loading={modelsLoading}
          error={modelsError}
        />
      )}

      {/* 简单模式：显示当前选择 */}
      {mode === "simple" && selectedModel && (
        <div className="p-3 rounded-lg bg-muted/50 border">
          <div className="text-sm font-medium">
            {selectedModel.display_name}
          </div>
          <div className="text-xs text-muted-foreground">
            {selectedModel.provider_type}
          </div>
        </div>
      )}

      {/* 错误提示 */}
      {error && (
        <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm">
          {error}
        </div>
      )}
    </div>
  );
}

// 导出子组件
export { TierSelector } from "./TierSelector";
export { ModeToggle } from "./ModeToggle";
export { ModelList } from "./ModelList";
export type { SelectionMode } from "./ModeToggle";
