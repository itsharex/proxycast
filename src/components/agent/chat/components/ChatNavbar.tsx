import React, { useState, useMemo, useEffect, useRef } from "react";
import { Bot, ChevronDown, Check, Box, Settings2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Navbar } from "../styles";
import { cn } from "@/lib/utils";
import { useProviderPool } from "@/hooks/useProviderPool";
import { useApiKeyProvider } from "@/hooks/useApiKeyProvider";
import { useModelRegistry } from "@/hooks/useModelRegistry";
import { getDefaultProvider } from "@/hooks/useTauri";
import { getProviderAliasConfig } from "@/lib/api/modelRegistry";
import type { ProviderAliasConfig } from "@/lib/types/modelRegistry";

// Provider type 到 registry ID 的映射（用于获取模型列表）
// 注意：antigravity 和 kiro 使用别名配置，需要单独处理
const getRegistryIdFromType = (providerType: string): string => {
  const typeMap: Record<string, string> = {
    openai: "openai",
    anthropic: "anthropic",
    gemini: "google",
    "azure-openai": "openai",
    vertexai: "google",
    ollama: "ollama",
    kiro: "kiro", // 使用别名配置
    claude: "anthropic",
    claude_oauth: "anthropic",
    qwen: "alibaba",
    codex: "openai",
    antigravity: "antigravity", // 使用别名配置
    iflow: "openai",
    gemini_api_key: "google",
  };
  return typeMap[providerType.toLowerCase()] || providerType.toLowerCase();
};

// 需要使用别名配置的 Provider 列表
const ALIAS_PROVIDERS = ["antigravity", "kiro"];

// 生成 Provider 的显示标签
const getProviderLabel = (providerType: string): string => {
  const labelMap: Record<string, string> = {
    kiro: "Kiro",
    gemini: "Gemini OAuth",
    qwen: "通义千问",
    antigravity: "Antigravity",
    codex: "Codex",
    claude_oauth: "Claude OAuth",
    claude: "Claude",
    openai: "OpenAI",
    anthropic: "Anthropic",
    "azure-openai": "Azure OpenAI",
    vertexai: "VertexAI",
    ollama: "Ollama",
    gemini_api_key: "Gemini API Key",
    iflow: "iFlow",
  };
  // 如果在映射表中，使用映射；否则首字母大写
  return (
    labelMap[providerType.toLowerCase()] ||
    providerType.charAt(0).toUpperCase() + providerType.slice(1)
  );
};

/** 已配置的 Provider 信息 */
interface ConfiguredProvider {
  key: string;
  label: string;
  registryId: string;
  fallbackRegistryId?: string; // 当 registryId 没有模型时的回退
  type: string; // 原始 provider type，用于确定 API 协议
}

interface ChatNavbarProps {
  providerType: string;
  setProviderType: (type: string) => void;
  model: string;
  setModel: (model: string) => void;
  isRunning: boolean;
  onToggleHistory: () => void;
  onToggleFullscreen: () => void;
  onToggleSettings?: () => void;
}

export const ChatNavbar: React.FC<ChatNavbarProps> = ({
  providerType,
  setProviderType,
  model,
  setModel,
  isRunning: _isRunning,
  onToggleHistory,
  onToggleFullscreen: _onToggleFullscreen,
  onToggleSettings,
}) => {
  const [open, setOpen] = useState(false);
  const [serverDefaultProvider, setServerDefaultProvider] = useState<
    string | null
  >(null);

  // 用于防止无限循环
  const hasInitialized = useRef(false);

  // 别名配置缓存（用于 Antigravity/Kiro 等中转服务）
  const [aliasConfig, setAliasConfig] = useState<ProviderAliasConfig | null>(
    null,
  );

  // 获取凭证池数据
  const { overview: oauthCredentials } = useProviderPool();
  const { providers: apiKeyProviders } = useApiKeyProvider();

  // 获取服务器默认 Provider
  useEffect(() => {
    const loadDefaultProvider = async () => {
      try {
        const dp = await getDefaultProvider();
        setServerDefaultProvider(dp);
      } catch (e) {
        console.error("Failed to get default provider:", e);
      }
    };
    loadDefaultProvider();
  }, []);

  // 获取模型注册表数据
  const { models: registryModels } = useModelRegistry({ autoLoad: true });

  // 计算已配置的 Provider 列表（完全动态，无白名单限制）
  const configuredProviders = useMemo(() => {
    const providerMap = new Map<string, ConfiguredProvider>();

    // 从 OAuth 凭证提取 Provider（动态，支持所有类型）
    oauthCredentials.forEach((overview) => {
      if (overview.credentials.length > 0) {
        const key = overview.provider_type;
        if (!providerMap.has(key)) {
          providerMap.set(key, {
            key,
            label: getProviderLabel(key),
            registryId: getRegistryIdFromType(key),
            type: key,
          });
        }
      }
    });

    // 从 API Key Provider 提取（动态，支持所有自定义 Provider）
    // 使用 provider.id 作为 key，确保每个 Provider 单独显示
    // 特殊处理：如果与 OAuth 凭证冲突，使用带后缀的 key
    apiKeyProviders
      .filter((p) => p.api_key_count > 0 && p.enabled)
      .forEach((provider) => {
        let key = provider.id;
        let label = provider.name;

        // 如果 key 与 OAuth 凭证冲突，添加 "_api_key" 后缀
        // 例如：Gemini OAuth 的 key 是 "gemini"，Gemini API Key 的 key 变成 "gemini_api_key"
        if (providerMap.has(key)) {
          key = `${provider.id}_api_key`;
          label = `${provider.name} API Key`;
        }

        if (!providerMap.has(key)) {
          // 优先使用 provider.id 作为 registryId（适用于系统预设的 Provider，如 deepseek, moonshot）
          // 如果模型注册表中没有该 id 的模型，则回退到使用 type 映射（适用于自定义 Provider）
          providerMap.set(key, {
            key,
            label,
            registryId: provider.id, // 先尝试用 id
            fallbackRegistryId: getRegistryIdFromType(provider.type), // 回退用 type
            type: provider.type,
          });
        }
      });

    return Array.from(providerMap.values());
  }, [oauthCredentials, apiKeyProviders]);

  // 获取当前选中 Provider 的配置
  const selectedProvider = useMemo(() => {
    return configuredProviders.find((p) => p.key === providerType);
  }, [configuredProviders, providerType]);

  // 当选中别名 Provider 时，加载别名配置
  useEffect(() => {
    if (selectedProvider && ALIAS_PROVIDERS.includes(selectedProvider.key)) {
      getProviderAliasConfig(selectedProvider.key)
        .then((config) => {
          setAliasConfig(config);
        })
        .catch((error) => {
          console.error("加载别名配置失败:", error);
          setAliasConfig(null);
        });
    } else {
      setAliasConfig(null);
    }
  }, [selectedProvider]);

  // 获取当前 Provider 的模型列表（从 model_registry 获取）
  // 按照模型版本排序，最新的在前面
  const currentModels = useMemo(() => {
    if (!selectedProvider) return [];

    // 对于别名 Provider（Antigravity、Kiro），使用别名配置中的模型列表
    if (ALIAS_PROVIDERS.includes(selectedProvider.key) && aliasConfig) {
      return aliasConfig.models;
    }

    // 从 model_registry 获取模型
    // 优先使用 registryId，如果没有模型则回退到 fallbackRegistryId
    let models = registryModels.filter(
      (m) => m.provider_id === selectedProvider.registryId,
    );

    // 如果没有找到模型，尝试使用 fallbackRegistryId
    if (models.length === 0 && selectedProvider.fallbackRegistryId) {
      models = registryModels.filter(
        (m) => m.provider_id === selectedProvider.fallbackRegistryId,
      );
    }

    // 按照 release_date 和 is_latest 排序
    const sortedModels = [...models].sort((a, b) => {
      // 1. is_latest 优先
      if (a.is_latest && !b.is_latest) return -1;
      if (!a.is_latest && b.is_latest) return 1;

      // 2. 按 release_date 降序（最新的在前）
      if (a.release_date && b.release_date) {
        return b.release_date.localeCompare(a.release_date);
      }
      if (a.release_date && !b.release_date) return -1;
      if (!a.release_date && b.release_date) return 1;

      // 3. 按 display_name 字母序
      return a.display_name.localeCompare(b.display_name);
    });

    return sortedModels.map((m) => m.id);
  }, [selectedProvider, registryModels, aliasConfig]);

  // 初始化：优先选择服务器默认 Provider，否则选择第一个已配置的
  useEffect(() => {
    if (hasInitialized.current) return;
    if (configuredProviders.length === 0) return;
    if (serverDefaultProvider === null) return; // 等待服务器默认 Provider 加载完成

    // 检查服务器默认 Provider 是否在已配置列表中
    const serverDefaultInList = configuredProviders.find(
      (p) => p.key === serverDefaultProvider,
    );

    if (serverDefaultInList) {
      // 服务器默认 Provider 在列表中，使用它
      hasInitialized.current = true;
      if (providerType !== serverDefaultProvider) {
        setProviderType(serverDefaultProvider);
      }
    } else if (!selectedProvider) {
      // 服务器默认 Provider 不在列表中，使用第一个已配置的
      hasInitialized.current = true;
      setProviderType(configuredProviders[0].key);
    } else {
      hasInitialized.current = true;
    }
  }, [
    configuredProviders,
    selectedProvider,
    setProviderType,
    serverDefaultProvider,
    providerType,
  ]);

  // 当 Provider 切换或模型列表变化时，自动选择第一个模型
  // 注意：使用 ref 跟踪 model 避免将其放入依赖中导致无限循环
  const modelRef = useRef(model);
  modelRef.current = model;

  useEffect(() => {
    // 对于别名 Provider，等待别名配置加载完成
    if (
      selectedProvider &&
      ALIAS_PROVIDERS.includes(selectedProvider.key) &&
      !aliasConfig
    ) {
      return;
    }

    // 如果模型列表不为空，且当前模型为空或不在列表中，选择第一个模型
    const currentModel = modelRef.current;
    if (
      currentModels.length > 0 &&
      (!currentModel || !currentModels.includes(currentModel))
    ) {
      setModel(currentModels[0]);
    }
  }, [currentModels, setModel, selectedProvider, aliasConfig]);

  const selectedProviderLabel = selectedProvider?.label || providerType;

  return (
    <Navbar>
      <div className="flex items-center gap-2">
        {/* History Toggle (Left) */}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground"
          onClick={onToggleHistory}
        >
          <Box size={18} />
        </Button>
      </div>

      {/* Center: Model Selector */}
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2">
        <Popover open={open} onOpenChange={setOpen}>
          <PopoverTrigger asChild>
            <Button
              variant="ghost"
              role="combobox"
              aria-expanded={open}
              className="h-9 px-3 gap-2 font-normal hover:bg-muted text-foreground"
            >
              <Bot size={16} className="text-primary" />
              <span className="font-medium">{selectedProviderLabel}</span>
              <span className="text-muted-foreground">/</span>
              <span className="text-sm">{model || "Select Model"}</span>
              <ChevronDown className="ml-1 h-3 w-3 text-muted-foreground opacity-50" />
            </Button>
          </PopoverTrigger>
          <PopoverContent
            className="w-[420px] p-0 bg-background/95 backdrop-blur-sm border-border shadow-lg"
            align="center"
          >
            {/* Provider/Model Selection */}
            <div className="flex h-[300px]">
              {/* Left Column: Providers (只显示已配置的) */}
              <div className="w-[140px] border-r bg-muted/30 p-2 flex flex-col gap-1 overflow-y-auto">
                <div className="text-xs font-semibold text-muted-foreground px-2 py-1.5 mb-1">
                  Providers
                </div>
                {configuredProviders.length === 0 ? (
                  <div className="text-xs text-muted-foreground p-2">
                    暂无已配置的 Provider
                  </div>
                ) : (
                  configuredProviders.map((provider) => {
                    // 判断是否是服务器默认 Provider
                    const isServerDefault =
                      serverDefaultProvider === provider.key;
                    const isSelected = providerType === provider.key;

                    return (
                      <button
                        key={provider.key}
                        onClick={() => {
                          setProviderType(provider.key);
                        }}
                        className={cn(
                          "flex items-center justify-between w-full px-2 py-1.5 text-sm rounded-md transition-colors text-left",
                          isSelected
                            ? "bg-primary/10 text-primary font-medium"
                            : isServerDefault
                              ? "hover:bg-muted text-foreground hover:text-foreground"
                              : "hover:bg-muted text-muted-foreground/50 hover:text-muted-foreground",
                        )}
                      >
                        {provider.label}
                        {isSelected && (
                          <div className="w-1 h-1 rounded-full bg-primary" />
                        )}
                      </button>
                    );
                  })
                )}
              </div>

              {/* Right Column: Models */}
              <div className="flex-1 p-2 flex flex-col overflow-hidden">
                <div className="text-xs font-semibold text-muted-foreground px-2 py-1.5 mb-1">
                  Models
                </div>
                <ScrollArea className="flex-1">
                  <div className="space-y-1 p-1">
                    {currentModels.length === 0 ? (
                      <div className="text-xs text-muted-foreground p-2">
                        No models available
                      </div>
                    ) : (
                      currentModels.map((m) => (
                        <button
                          key={m}
                          onClick={() => {
                            setModel(m);
                            setOpen(false);
                          }}
                          className={cn(
                            "flex items-center justify-between w-full px-2 py-1.5 text-sm rounded-md transition-colors text-left group",
                            model === m
                              ? "bg-accent text-accent-foreground"
                              : "hover:bg-muted text-muted-foreground hover:text-foreground",
                          )}
                        >
                          {m}
                          {model === m && (
                            <Check size={14} className="text-primary" />
                          )}
                        </button>
                      ))
                    )}
                  </div>
                </ScrollArea>
              </div>
            </div>
          </PopoverContent>
        </Popover>
      </div>

      {/* Right: Status & Settings */}
      <div className="flex items-center gap-2">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-muted-foreground"
          onClick={onToggleSettings}
        >
          <Settings2 size={18} />
        </Button>
      </div>
    </Navbar>
  );
};
