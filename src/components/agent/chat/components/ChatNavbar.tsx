import React, { useState, useEffect } from "react";
import {
  Bot,
  ChevronDown,
  Check,
  Box,
  Settings2,
  Zap,
  Sparkles,
  Crown,
  Wand2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { Navbar } from "../styles";
import { PROVIDER_CONFIG } from "../types";
import { cn } from "@/lib/utils";
import {
  orchestratorApi,
  type ServiceTier,
  type PoolStats,
} from "@/lib/api/orchestrator";

// 服务等级配置
const TIER_CONFIG: Record<
  ServiceTier,
  {
    label: string;
    description: string;
    icon: React.ReactNode;
    color: string;
    bgColor: string;
  }
> = {
  mini: {
    label: "Mini",
    description: "快速响应",
    icon: <Zap className="w-3.5 h-3.5" />,
    color: "text-green-600 dark:text-green-400",
    bgColor: "bg-green-500/10",
  },
  pro: {
    label: "Pro",
    description: "均衡性能",
    icon: <Sparkles className="w-3.5 h-3.5" />,
    color: "text-blue-600 dark:text-blue-400",
    bgColor: "bg-blue-500/10",
  },
  max: {
    label: "Max",
    description: "最强能力",
    icon: <Crown className="w-3.5 h-3.5" />,
    color: "text-purple-600 dark:text-purple-400",
    bgColor: "bg-purple-500/10",
  },
};

type SelectionMode = "simple" | "expert";

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
  const [mode, setMode] = useState<SelectionMode>("simple");
  const [tier, setTier] = useState<ServiceTier>("pro");
  const [poolStats, setPoolStats] = useState<PoolStats | null>(null);
  const [orchestratorReady, setOrchestratorReady] = useState(false);

  // 初始化 orchestrator
  useEffect(() => {
    const init = async () => {
      try {
        await orchestratorApi.init();
        setOrchestratorReady(true);
        const stats = await orchestratorApi.getPoolStats();
        setPoolStats(stats);
      } catch (err) {
        console.warn("Orchestrator 初始化失败，使用专家模式:", err);
        setMode("expert");
      }
    };
    init();
  }, []);

  // 简单模式下选择等级时自动选择模型
  const handleTierSelect = async (selectedTier: ServiceTier) => {
    setTier(selectedTier);
    setOpen(false);

    if (!orchestratorReady) return;

    try {
      const result = await orchestratorApi.selectModel({ tier: selectedTier });
      // 映射 orchestrator 的 provider_type 到 PROVIDER_CONFIG 的 key
      const providerKey = mapProviderType(result.provider_type);
      setProviderType(providerKey);
      setModel(result.model_id);
    } catch (err) {
      console.error("模型选择失败:", err);
    }
  };

  // 映射 provider type
  const mapProviderType = (orchestratorType: string): string => {
    const mapping: Record<string, string> = {
      anthropic: "claude",
      openai: "openai",
      google: "gemini",
      gemini: "gemini",
      kiro: "kiro",
      codex: "codex",
    };
    return mapping[orchestratorType.toLowerCase()] || orchestratorType;
  };

  const selectedProviderLabel =
    PROVIDER_CONFIG[providerType]?.label || providerType;
  const currentModels = PROVIDER_CONFIG[providerType]?.models || [];
  const tierConfig = TIER_CONFIG[tier];

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
              {mode === "simple" && orchestratorReady ? (
                <>
                  <span className={tierConfig.color}>{tierConfig.icon}</span>
                  <span className="font-medium">{tierConfig.label}</span>
                  <span className="text-muted-foreground text-xs">
                    ({tierConfig.description})
                  </span>
                </>
              ) : (
                <>
                  <Bot size={16} className="text-primary" />
                  <span className="font-medium">{selectedProviderLabel}</span>
                  <span className="text-muted-foreground">/</span>
                  <span className="text-sm">{model || "Select Model"}</span>
                </>
              )}
              <ChevronDown className="ml-1 h-3 w-3 text-muted-foreground opacity-50" />
            </Button>
          </PopoverTrigger>
          <PopoverContent
            className="w-[420px] p-0 bg-background/95 backdrop-blur-sm border-border shadow-lg"
            align="center"
          >
            {/* Mode Toggle */}
            <div className="flex items-center justify-between px-3 py-2 border-b bg-muted/30">
              <span className="text-xs font-medium text-muted-foreground">
                选择模式
              </span>
              <div className="flex gap-1">
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={mode === "simple" ? "secondary" : "ghost"}
                      size="sm"
                      className="h-7 px-2 text-xs"
                      onClick={() => setMode("simple")}
                      disabled={!orchestratorReady}
                    >
                      <Wand2 className="w-3 h-3 mr-1" />
                      简单
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>Mini/Pro/Max 三档智能选择</TooltipContent>
                </Tooltip>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant={mode === "expert" ? "secondary" : "ghost"}
                      size="sm"
                      className="h-7 px-2 text-xs"
                      onClick={() => setMode("expert")}
                    >
                      <Settings2 className="w-3 h-3 mr-1" />
                      专家
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>直接选择 Provider 和模型</TooltipContent>
                </Tooltip>
              </div>
            </div>

            {mode === "simple" && orchestratorReady ? (
              /* Simple Mode: Tier Selection */
              <div className="p-3">
                <div className="grid grid-cols-3 gap-2">
                  {(Object.keys(TIER_CONFIG) as ServiceTier[]).map((t) => {
                    const config = TIER_CONFIG[t];
                    const count =
                      poolStats?.[`${t}_count` as keyof PoolStats] ?? 0;
                    const isSelected = tier === t;

                    return (
                      <button
                        key={t}
                        onClick={() => handleTierSelect(t)}
                        className={cn(
                          "flex flex-col items-center p-3 rounded-lg border transition-all",
                          isSelected
                            ? cn(
                                "border-primary/50",
                                config.bgColor,
                                "ring-2 ring-primary/20",
                              )
                            : "border-border hover:bg-muted/50",
                          count === 0 && "opacity-50 cursor-not-allowed",
                        )}
                        disabled={count === 0}
                      >
                        <span
                          className={cn(
                            "mb-1",
                            isSelected ? config.color : "text-muted-foreground",
                          )}
                        >
                          {config.icon}
                        </span>
                        <span
                          className={cn(
                            "font-medium text-sm",
                            isSelected ? config.color : "text-foreground",
                          )}
                        >
                          {config.label}
                        </span>
                        <span className="text-xs text-muted-foreground">
                          {config.description}
                        </span>
                        {poolStats && (
                          <span className="text-xs text-muted-foreground mt-1">
                            {count} 模型
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>

                {/* Current Selection Info */}
                {model && (
                  <div className="mt-3 p-2 rounded-md bg-muted/50 text-xs">
                    <span className="text-muted-foreground">当前模型: </span>
                    <span className="font-medium">
                      {selectedProviderLabel} / {model}
                    </span>
                  </div>
                )}
              </div>
            ) : (
              /* Expert Mode: Provider/Model Selection */
              <div className="flex h-[300px]">
                {/* Left Column: Providers */}
                <div className="w-[140px] border-r bg-muted/30 p-2 flex flex-col gap-1 overflow-y-auto">
                  <div className="text-xs font-semibold text-muted-foreground px-2 py-1.5 mb-1">
                    Providers
                  </div>
                  {Object.entries(PROVIDER_CONFIG).map(([key, config]) => (
                    <button
                      key={key}
                      onClick={() => {
                        setProviderType(key);
                        // Auto-select first model if available
                        if (config.models.length > 0) {
                          setModel(config.models[0]);
                        } else {
                          setModel("");
                        }
                      }}
                      className={cn(
                        "flex items-center justify-between w-full px-2 py-1.5 text-sm rounded-md transition-colors text-left",
                        providerType === key
                          ? "bg-primary/10 text-primary font-medium"
                          : "hover:bg-muted text-muted-foreground hover:text-foreground",
                      )}
                    >
                      {config.label}
                      {providerType === key && (
                        <div className="w-1 h-1 rounded-full bg-primary" />
                      )}
                    </button>
                  ))}
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
            )}
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
