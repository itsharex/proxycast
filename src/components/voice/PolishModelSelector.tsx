/**
 * @file PolishModelSelector.tsx
 * @description 语音润色模型选择器 - 用于语音设置中选择润色使用的模型
 * @module components/voice/PolishModelSelector
 */

import React, {
  useState,
  useRef,
  useEffect,
  useMemo,
  useCallback,
} from "react";
import {
  ChevronDown,
  Loader2,
  Check,
  Sparkles,
  ChevronRight,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useConfiguredProviders } from "@/hooks/useConfiguredProviders";
import { useProviderModels } from "@/hooks/useProviderModels";
import { getProviderLabel } from "@/lib/constants/providerMappings";
import { getProviderModelCompatibilityIssue } from "@/components/agent/chat/utils/providerModelCompatibility";

export interface PolishModelSelectorProps {
  /** 当前选中的模型 ID */
  value?: string;
  /** 模型变更回调 */
  onChange: (modelId: string) => void;
  /** 是否禁用 */
  disabled?: boolean;
  /** 自定义类名 */
  className?: string;
}

/**
 * 语音润色模型选择器
 *
 * 允许用户选择用于语音润色的 LLM 模型
 */
export const PolishModelSelector: React.FC<PolishModelSelectorProps> = ({
  value,
  onChange,
  disabled = false,
  className,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedProviderKey, setSelectedProviderKey] = useState<string | null>(
    null,
  );
  const menuRef = useRef<HTMLDivElement>(null);

  // 获取已配置的 Provider 列表
  const { providers, loading: providersLoading } = useConfiguredProviders();

  // 获取当前选中的 Provider 对象
  const selectedProvider = useMemo(() => {
    return providers.find((p) => p.key === selectedProviderKey) || null;
  }, [providers, selectedProviderKey]);

  // 获取当前 Provider 的模型列表（传入完整的 Provider 对象）
  const { models: currentModels, loading: modelsLoading } = useProviderModels(
    selectedProvider,
    { returnFullMetadata: true },
  );

  const compatibleModels = useMemo(
    () =>
      currentModels.filter(
        (model) =>
          !getProviderModelCompatibilityIssue({
            providerType: selectedProvider?.key || "",
            configuredProviderType: selectedProvider?.type,
            model: model.id,
          }),
      ),
    [currentModels, selectedProvider?.key, selectedProvider?.type],
  );

  // 点击外部关闭
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }

    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [isOpen]);

  // 初始化选中的 Provider（根据当前值或默认第一个）
  useEffect(() => {
    if (providers.length > 0 && !selectedProviderKey) {
      // 默认选择第一个 Provider
      setSelectedProviderKey(providers[0].key);
    }
  }, [providers, selectedProviderKey]);

  // 获取显示文本
  const displayText = useMemo(() => {
    if (providersLoading) {
      return "加载中...";
    }
    if (!value) {
      return "选择润色模型";
    }
    // 简化模型名称显示
    const shortModelId = value.split("/").pop() || value;
    return shortModelId;
  }, [providersLoading, value]);

  // 处理 Provider 选择
  const handleProviderSelect = useCallback((providerKey: string) => {
    setSelectedProviderKey(providerKey);
  }, []);

  // 处理模型选择
  const handleModelSelect = useCallback(
    (modelId: string) => {
      onChange(modelId);
      setIsOpen(false);
    },
    [onChange],
  );

  const isLoading = providersLoading || modelsLoading;

  return (
    <div className={cn("relative", className)} ref={menuRef}>
      {/* 触发按钮 */}
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled || isLoading}
        className={cn(
          "w-full flex items-center justify-between gap-2 px-3 py-2 text-sm rounded-md transition-colors",
          "border border-border hover:border-border hover:bg-muted",
          "focus:outline-none focus:ring-2 focus:ring-primary/50",
          disabled && "opacity-50 cursor-not-allowed",
          isOpen && "border-primary bg-primary/5",
        )}
      >
        <div className="flex items-center gap-2 min-w-0">
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin text-muted-foreground flex-shrink-0" />
          ) : (
            <Sparkles className="h-4 w-4 text-muted-foreground flex-shrink-0" />
          )}
          <span className="truncate">{displayText}</span>
        </div>
        <ChevronDown
          className={cn(
            "h-4 w-4 text-muted-foreground transition-transform flex-shrink-0",
            isOpen && "rotate-180",
          )}
        />
      </button>

      {/* 下拉菜单 */}
      {isOpen && (
        <div className="absolute top-full left-0 mt-1 w-full min-w-[400px] max-h-80 overflow-hidden rounded-lg border border-border shadow-lg z-50 bg-white dark:bg-zinc-900">
          <div className="flex h-72">
            {/* 左侧：Provider 列表 */}
            <div className="w-36 border-r border-border flex flex-col bg-zinc-50 dark:bg-zinc-800">
              <div className="px-3 py-2 border-b border-border bg-zinc-100 dark:bg-zinc-700">
                <h4 className="text-xs font-medium text-foreground">
                  Provider
                </h4>
              </div>
              <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
                {providers.map((provider) => (
                  <button
                    key={provider.key}
                    type="button"
                    onClick={() => handleProviderSelect(provider.key)}
                    className={cn(
                      "w-full flex items-center justify-between px-2 py-1.5 text-xs rounded transition-colors",
                      selectedProviderKey === provider.key
                        ? "bg-primary text-primary-foreground"
                        : "hover:bg-muted text-foreground",
                    )}
                  >
                    <span className="truncate">{provider.label}</span>
                    <ChevronRight
                      className={cn(
                        "h-3 w-3 flex-shrink-0 transition-transform",
                        selectedProviderKey === provider.key && "rotate-90",
                      )}
                    />
                  </button>
                ))}
              </div>
            </div>

            {/* 右侧：模型列表 */}
            <div className="flex-1 flex flex-col min-w-0 bg-white dark:bg-zinc-900">
              <div className="px-3 py-2 border-b border-border bg-zinc-100 dark:bg-zinc-700">
                <h4 className="text-xs font-medium text-foreground">
                  {selectedProvider
                    ? `${getProviderLabel(selectedProvider.key)} 模型`
                    : "请选择 Provider"}
                </h4>
              </div>
              <div className="flex-1 overflow-y-auto p-1.5 space-y-0.5">
                {modelsLoading ? (
                  <div className="flex items-center justify-center h-full">
                    <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
                  </div>
                ) : compatibleModels.length === 0 ? (
                  <div className="flex items-center justify-center h-full text-xs text-muted-foreground">
                    暂无可用模型
                  </div>
                ) : (
                  compatibleModels.map((model) => {
                    const isSelected = value === model.id;
                    // 简化模型名称显示
                    const displayName =
                      model.display_name ||
                      model.id.split("/").pop() ||
                      model.id;

                    return (
                      <button
                        key={model.id}
                        type="button"
                        onClick={() => handleModelSelect(model.id)}
                        className={cn(
                          "w-full flex items-center justify-between px-2 py-1.5 text-xs rounded transition-colors",
                          isSelected
                            ? "bg-primary/10 text-primary border border-primary/30"
                            : "hover:bg-muted text-foreground border border-transparent",
                        )}
                      >
                        <span className="truncate">{displayName}</span>
                        {isSelected && (
                          <Check className="h-3 w-3 flex-shrink-0" />
                        )}
                      </button>
                    );
                  })
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default PolishModelSelector;
