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
import { getDefaultProvider } from "@/hooks/useTauri";
import { useConfiguredProviders } from "@/hooks/useConfiguredProviders";
import { useProviderModels } from "@/hooks/useProviderModels";
import { isAliasProvider } from "@/lib/constants/providerMappings";
import { providerPoolApi } from "@/lib/api/providerPool";
import { apiKeyProviderApi } from "@/lib/api/apiKeyProvider";
import { emitProviderDataChanged } from "@/lib/providerDataEvents";

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

  // 获取已配置的 Provider 列表（使用共享 hook）
  const { providers: configuredProviders } = useConfiguredProviders();

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

  // 获取当前选中 Provider 的配置
  const selectedProvider = useMemo(() => {
    return configuredProviders.find((p) => p.key === providerType);
  }, [configuredProviders, providerType]);

  // 获取当前 Provider 的模型列表（使用共享 hook）
  const { modelIds: currentModels, loading: modelsLoading } =
    useProviderModels(selectedProvider);

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
    // 对于别名 Provider，等待模型加载完成
    if (
      selectedProvider &&
      isAliasProvider(selectedProvider.key) &&
      modelsLoading
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
  }, [currentModels, setModel, selectedProvider, modelsLoading]);

  const selectedProviderLabel = selectedProvider?.label || providerType;

  // 当打开模型选择器时，主动触发一次 Provider 数据同步
  useEffect(() => {
    if (!open) return;

    let cancelled = false;

    const refreshProviderData = async () => {
      try {
        await Promise.all([
          providerPoolApi.getOverview(),
          apiKeyProviderApi.getProviders(),
        ]);

        if (!cancelled) {
          emitProviderDataChanged("provider_pool");
        }
      } catch (e) {
        console.error("[ChatNavbar] 刷新 Provider 数据失败:", e);
      }
    };

    void refreshProviderData();

    return () => {
      cancelled = true;
    };
  }, [open]);

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
