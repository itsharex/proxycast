/**
 * @file ProviderModelList 组件
 * @description 显示 Provider 支持的模型列表，支持从 API 刷新
 * @module components/provider-pool/api-key/ProviderModelList
 */

import React, { useMemo, useState, useCallback, useEffect } from "react";
import { cn } from "@/lib/utils";
import { useModelRegistry } from "@/hooks/useModelRegistry";
import {
  Eye,
  Wrench,
  Brain,
  Sparkles,
  Loader2,
  RefreshCw,
  Cloud,
  HardDrive,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import type { EnhancedModelMetadata } from "@/lib/types/modelRegistry";
import { apiKeyProviderApi } from "@/lib/api/apiKeyProvider";
import { modelRegistryApi } from "@/lib/api/modelRegistry";
import {
  buildCatalogAliasMap,
  resolveRegistryProviderId,
} from "./providerTypeMapping";
import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// 类型定义
// ============================================================================

export interface ProviderModelListProps {
  /** Provider ID，如 "deepseek", "openai", "anthropic" */
  providerId: string;
  /** Provider 类型（API 协议），如 "anthropic", "openai", "gemini" */
  providerType: string;
  /** 是否有可用的 API Key（用于显示刷新按钮） */
  hasApiKey?: boolean;
  /** 额外的 CSS 类名 */
  className?: string;
  /** 最大显示数量，默认显示全部 */
  maxItems?: number;
}

// ============================================================================
// API 响应类型
// ============================================================================

interface FetchModelsResult {
  models: EnhancedModelMetadata[];
  source: "Api" | "LocalFallback";
  error: string | null;
}

// ============================================================================
// 子组件
// ============================================================================

interface ModelItemProps {
  model: EnhancedModelMetadata;
}

/**
 * 单个模型项
 */
const ModelItem: React.FC<ModelItemProps> = ({ model }) => {
  return (
    <div
      className="flex items-center justify-between py-2 px-3 rounded-md hover:bg-muted/50 transition-colors"
      data-testid={`model-item-${model.id}`}
    >
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium truncate">
            {model.display_name}
          </span>
        </div>
        <div className="text-xs text-muted-foreground truncate">{model.id}</div>
      </div>

      {/* 能力标签 */}
      <div className="flex items-center gap-1.5 ml-2">
        {model.capabilities.vision && (
          <span
            className="text-blue-500"
            title="支持视觉"
            data-testid="capability-vision"
          >
            <Eye className="h-3.5 w-3.5" />
          </span>
        )}
        {model.capabilities.tools && (
          <span
            className="text-orange-500"
            title="支持工具调用"
            data-testid="capability-tools"
          >
            <Wrench className="h-3.5 w-3.5" />
          </span>
        )}
        {model.capabilities.reasoning && (
          <span
            className="text-purple-500"
            title="支持推理"
            data-testid="capability-reasoning"
          >
            <Brain className="h-3.5 w-3.5" />
          </span>
        )}
      </div>
    </div>
  );
};

// ============================================================================
// 主组件
// ============================================================================

/**
 * Provider 支持的模型列表组件
 *
 * 显示指定 Provider 支持的所有模型，包括模型名称和能力标签
 *
 * @example
 * ```tsx
 * <ProviderModelList providerType="anthropic" />
 * ```
 */
export const ProviderModelList: React.FC<ProviderModelListProps> = ({
  providerId,
  providerType,
  hasApiKey = false,
  className,
  maxItems,
}) => {
  const [catalogAliasMap, setCatalogAliasMap] = useState<Record<
    string,
    string
  > | null>(null);
  const [validRegistryProviderIds, setValidRegistryProviderIds] =
    useState<Set<string> | null>(null);

  useEffect(() => {
    let cancelled = false;

    const loadCatalog = async () => {
      try {
        const catalog = await apiKeyProviderApi.getSystemProviderCatalog();
        if (cancelled) {
          return;
        }
        setCatalogAliasMap(buildCatalogAliasMap(catalog));
      } catch {
        if (cancelled) {
          return;
        }
        setCatalogAliasMap(null);
      }
    };

    loadCatalog();

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    let cancelled = false;

    const loadRegistryProviders = async () => {
      try {
        const providerIds =
          await modelRegistryApi.getModelRegistryProviderIds();
        if (cancelled) {
          return;
        }

        setValidRegistryProviderIds(new Set(providerIds));
      } catch {
        if (cancelled) {
          return;
        }
        setValidRegistryProviderIds(null);
      }
    };

    loadRegistryProviders();

    return () => {
      cancelled = true;
    };
  }, []);

  // 转换 Provider ID 为 registry ID（优先使用 providerId，回退到 providerType）
  const registryProviderId = useMemo(() => {
    return resolveRegistryProviderId(providerId, {
      providerType,
      catalogAliasMap,
      validRegistryProviders: validRegistryProviderIds ?? undefined,
    });
  }, [catalogAliasMap, providerId, providerType, validRegistryProviderIds]);

  // 获取模型数据
  const { models, loading, error } = useModelRegistry({
    autoLoad: true,
    providerFilter: [registryProviderId],
  });

  // 从 API 刷新状态
  const [refreshing, setRefreshing] = useState(false);
  const [apiModels, setApiModels] = useState<EnhancedModelMetadata[] | null>(
    null,
  );
  const [apiSource, setApiSource] = useState<"Api" | "LocalFallback" | null>(
    null,
  );
  const [apiError, setApiError] = useState<string | null>(null);

  // 从 API 获取模型列表（自动获取 API Key）
  const handleRefreshFromApi = useCallback(async () => {
    setRefreshing(true);
    setApiError(null);

    try {
      const result = await invoke<FetchModelsResult>(
        "fetch_provider_models_auto",
        {
          providerId,
        },
      );

      if (result && result.models) {
        setApiModels(result.models);
        setApiSource(result.source);
        if (result.error) {
          setApiError(result.error);
        }
      } else {
        setApiError("返回结果格式错误");
      }
    } catch (err) {
      setApiError(err instanceof Error ? err.message : String(err));
    } finally {
      setRefreshing(false);
    }
  }, [providerId]);

  // 使用 API 模型或本地模型
  const displayModelsSource = apiModels ?? models;

  // 限制显示数量
  const displayModels = useMemo(() => {
    if (maxItems && maxItems > 0) {
      return displayModelsSource.slice(0, maxItems);
    }
    return displayModelsSource;
  }, [displayModelsSource, maxItems]);

  const hasMore = maxItems && displayModelsSource.length > maxItems;

  // 加载状态
  if (loading && !apiModels) {
    return (
      <div
        className={cn(
          "flex items-center justify-center py-8 text-muted-foreground",
          className,
        )}
        data-testid="provider-model-list-loading"
      >
        <Loader2 className="h-4 w-4 animate-spin mr-2" />
        <span className="text-sm">加载模型列表...</span>
      </div>
    );
  }

  // 错误状态
  if (error && !apiModels) {
    return (
      <div
        className={cn("py-4 text-center text-sm text-red-500", className)}
        data-testid="provider-model-list-error"
      >
        加载失败: {error}
      </div>
    );
  }

  // 空状态
  if (displayModelsSource.length === 0) {
    return (
      <div className={cn("space-y-2", className)}>
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-medium text-foreground flex items-center gap-2">
            <Sparkles className="h-4 w-4 text-muted-foreground" />
            支持的模型
          </h4>
          {hasApiKey && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleRefreshFromApi}
                    disabled={refreshing}
                    className="h-7 px-2"
                  >
                    {refreshing ? (
                      <Loader2 className="h-3.5 w-3.5 animate-spin" />
                    ) : (
                      <RefreshCw className="h-3.5 w-3.5" />
                    )}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>从 API 获取模型列表</TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </div>
        <div
          className="py-4 text-center text-sm text-muted-foreground"
          data-testid="provider-model-list-empty"
        >
          暂无模型数据
          {hasApiKey && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleRefreshFromApi}
              disabled={refreshing}
              className="ml-1 h-auto p-0 text-primary underline-offset-4 hover:underline"
            >
              点击从 API 获取
            </Button>
          )}
        </div>
        {apiError && (
          <div className="text-xs text-amber-500 text-center">{apiError}</div>
        )}
      </div>
    );
  }

  return (
    <div
      className={cn("space-y-1", className)}
      data-testid="provider-model-list"
    >
      {/* 标题 */}
      <div className="flex items-center justify-between mb-2">
        <h4 className="text-sm font-medium text-foreground flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-muted-foreground" />
          支持的模型
          <span className="text-xs text-muted-foreground font-normal">
            ({displayModelsSource.length})
          </span>
          {/* 数据来源标识 */}
          {apiSource && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span
                    className={cn(
                      "inline-flex items-center gap-1 text-xs px-1.5 py-0.5 rounded",
                      apiSource === "Api"
                        ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                        : "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400",
                    )}
                  >
                    {apiSource === "Api" ? (
                      <>
                        <Cloud className="h-3 w-3" />
                        API
                      </>
                    ) : (
                      <>
                        <HardDrive className="h-3 w-3" />
                        本地
                      </>
                    )}
                  </span>
                </TooltipTrigger>
                <TooltipContent>
                  {apiSource === "Api"
                    ? "数据来自 Provider API"
                    : "API 获取失败，使用本地数据"}
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
        </h4>
        {/* 刷新按钮 */}
        {hasApiKey && (
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleRefreshFromApi}
                  disabled={refreshing}
                  className="h-7 px-2"
                >
                  {refreshing ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3.5 w-3.5" />
                  )}
                </Button>
              </TooltipTrigger>
              <TooltipContent>从 API 获取最新模型列表</TooltipContent>
            </Tooltip>
          </TooltipProvider>
        )}
      </div>

      {/* API 错误提示 */}
      {apiError && (
        <div className="text-xs text-amber-500 mb-2 px-1">{apiError}</div>
      )}

      {/* 模型列表 */}
      <div className="border rounded-md divide-y divide-border">
        {displayModels.map((model) => (
          <ModelItem key={model.id} model={model} />
        ))}
      </div>

      {/* 显示更多提示 */}
      {hasMore && (
        <p className="text-xs text-muted-foreground text-center pt-2">
          还有 {displayModelsSource.length - maxItems!} 个模型未显示
        </p>
      )}
    </div>
  );
};

export default ProviderModelList;
