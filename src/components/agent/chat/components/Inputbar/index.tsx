import React, { useState, useCallback, useRef, useEffect, useMemo } from "react";
import { InputbarCore } from "./components/InputbarCore";
import { CharacterMention } from "./components/CharacterMention";
import { toast } from "sonner";
import styled from "styled-components";
import type { MessageImage } from "../../types";
import type { Character } from "@/lib/api/memory";
import type { Skill } from "@/lib/api/skills";
import { TaskFileList, type TaskFile } from "../TaskFiles";
import {
  FolderOpen,
  ChevronUp,
  ChevronDown,
  Code2,
  Loader2,
  Clock3,
  AlertCircle,
  Sparkles,
} from "lucide-react";
import { useActiveSkill } from "./hooks/useActiveSkill";
import { SkillBadge } from "./components/SkillBadge";
import { ChatModelSelector } from "../ChatModelSelector";
import { safeInvoke } from "@/lib/dev-bridge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from "@/components/ui/select";
import { createAgentInputAdapter } from "@/components/input-kit";
import type { StepStatus } from "@/components/content-creator/types";

// 任务文件触发器区域（在输入框上方，与输入框对齐）
const TaskFilesArea = styled.div`
  display: flex;
  justify-content: flex-end;
  padding: 0 8px 8px 8px;
  width: 100%;
  max-width: none;
  margin: 0;
`;

// 按钮和面板的包装容器
const TaskFilesWrapper = styled.div`
  position: relative;
`;

// 任务文件按钮
const TaskFilesButton = styled.button<{
  $expanded?: boolean;
  $hasFiles?: boolean;
}>`
  display: ${(props) => (props.$hasFiles ? "flex" : "none")};
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  font-size: 13px;
  color: hsl(var(--muted-foreground));
  cursor: pointer;
  transition: all 0.15s;

  &:hover {
    border-color: hsl(var(--primary) / 0.5);
    color: hsl(var(--foreground));
  }

  ${(props) =>
    props.$expanded &&
    `
    border-color: hsl(var(--primary));
    color: hsl(var(--foreground));
    background: hsl(var(--primary) / 0.05);
  `}
`;

const FileCount = styled.span`
  font-weight: 500;
`;

const ChevronIcon = styled.span<{ $expanded?: boolean }>`
  display: flex;
  align-items: center;
  transform: ${(props) =>
    props.$expanded ? "rotate(0deg)" : "rotate(180deg)"};
  transition: transform 0.2s;
`;

// Hint 路由弹出框
const HintPopup = styled.div`
  position: absolute;
  bottom: 100%;
  left: 8px;
  margin-bottom: 4px;
  background: hsl(var(--popover));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  padding: 4px;
  min-width: 180px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 50;
`;

const HintItem = styled.button<{ $active?: boolean }>`
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 6px 10px;
  border: none;
  border-radius: 6px;
  background: ${(props) =>
    props.$active ? "hsl(var(--accent))" : "transparent"};
  color: hsl(var(--foreground));
  cursor: pointer;
  text-align: left;
  font-size: 13px;
  line-height: 1.4;

  &:hover {
    background: hsl(var(--accent));
  }
`;

const HintLabel = styled.span`
  font-weight: 500;
`;

const HintModel = styled.span`
  font-size: 11px;
  color: hsl(var(--muted-foreground));
`;

const NOOP_SET_PROVIDER_TYPE = (_type: string) => {};
const NOOP_SET_MODEL = (_model: string) => {};
const SOCIAL_ARTICLE_SKILL_KEY = "social_post_with_cover";

const ThemeWorkbenchGateStrip = styled.div`
  margin: 0 12px 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px 10px;
  padding: 8px 10px;
  border-radius: 14px;
  border: 1px solid hsl(var(--border) / 0.92);
  background: hsl(var(--muted) / 0.78);
  box-shadow: none;
  opacity: 1;

  @media (prefers-color-scheme: dark) {
    background: hsl(222 18% 14% / 0.96);
    border-color: hsl(217 18% 24% / 0.95);
  }
`;

const ThemeWorkbenchGateMeta = styled.div`
  min-width: 0;
  display: inline-flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
`;

const ThemeWorkbenchGateIcon = styled.span`
  width: 22px;
  height: 22px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: hsl(var(--background));
  color: hsl(var(--muted-foreground));
  border: 1px solid hsl(var(--border) / 0.9);
  flex-shrink: 0;
`;

const ThemeWorkbenchGateTitle = styled.span`
  font-size: 12px;
  color: hsl(var(--foreground) / 0.86);
  font-weight: 600;
  line-height: 1.4;
`;

const ThemeWorkbenchGateStatus = styled.span<{
  $status: "running" | "waiting" | "idle";
}>`
  font-size: 11px;
  line-height: 1;
  border-radius: 999px;
  padding: 4px 8px;
  color: ${({ $status }) =>
    $status === "waiting"
      ? "hsl(var(--destructive))"
      : $status === "running"
        ? "hsl(var(--primary))"
        : "hsl(var(--muted-foreground))"};
  background: ${({ $status }) =>
    $status === "waiting"
      ? "hsl(var(--destructive) / 0.08)"
      : $status === "running"
        ? "hsl(var(--primary) / 0.1)"
        : "hsl(var(--muted) / 0.7)"};
`;

const ThemeWorkbenchQuickActions = styled.div`
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-left: auto;
`;

const ThemeWorkbenchQuickButton = styled.button`
  border: 1px solid hsl(var(--border) / 0.88);
  border-radius: 999px;
  background: hsl(var(--background));
  color: hsl(var(--foreground) / 0.82);
  font-size: 11px;
  line-height: 1.2;
  padding: 5px 10px;
  cursor: pointer;

  &:hover {
    border-color: hsl(var(--primary) / 0.22);
    color: hsl(var(--foreground));
    background: hsl(var(--background));
  }
`;

const ThemeWorkbenchGeneratingWrap = styled.div`
  margin: 0 10px 10px;
  display: flex;
  flex-direction: column;
  gap: 10px;
`;

const ThemeWorkbenchTaskCard = styled.div`
  border: 1px solid hsl(var(--border) / 0.78);
  border-radius: 15px;
  background: hsl(var(--background));
  box-shadow: 0 8px 20px hsl(var(--foreground) / 0.05);
  padding: 11px 12px 10px;
`;

const ThemeWorkbenchTaskHead = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  font-weight: 500;
  color: hsl(var(--muted-foreground));
  margin-bottom: 8px;
`;

const ThemeWorkbenchTaskHeadButton = styled.button`
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: none;
  background: transparent;
  color: inherit;
  padding: 0;
  cursor: pointer;
`;

const ThemeWorkbenchTaskHeadChevron = styled.span<{ $collapsed: boolean }>`
  display: inline-flex;
  transition: transform 0.2s ease;
  transform: ${({ $collapsed }) =>
    $collapsed ? "rotate(-90deg)" : "rotate(0deg)"};
`;

const ThemeWorkbenchTaskList = styled.div`
  display: flex;
  flex-direction: column;
  gap: 8px;
`;

const ThemeWorkbenchTaskRow = styled.div`
  display: flex;
  align-items: center;
  gap: 10px;
  min-height: 34px;
  min-width: 0;
`;

const ThemeWorkbenchTaskIcon = styled.span<{ $kind: "active" | "pending" | "error" }>`
  width: 30px;
  height: 30px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: ${({ $kind }) =>
    $kind === "active"
      ? "hsl(var(--primary) / 0.12)"
      : $kind === "error"
        ? "hsl(var(--destructive) / 0.1)"
        : "hsl(38 100% 92%)"};
  color: ${({ $kind }) =>
    $kind === "active"
      ? "hsl(var(--primary))"
      : $kind === "error"
        ? "hsl(var(--destructive))"
        : "hsl(30 90% 42%)"};
  flex-shrink: 0;
`;

const ThemeWorkbenchTaskText = styled.span`
  flex: 1;
  font-size: 14px;
  color: hsl(var(--foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const ThemeWorkbenchTaskStatus = styled.span<{ $kind: "active" | "pending" | "error" }>`
  font-size: 11px;
  border-radius: 999px;
  padding: 4px 10px;
  line-height: 1;
  font-weight: 600;
  color: ${(props) =>
    props.$kind === "active"
      ? "hsl(var(--primary))"
      : props.$kind === "error"
        ? "hsl(var(--destructive))"
        : "hsl(35 95% 35%)"};
  background: ${(props) =>
    props.$kind === "active"
      ? "hsl(var(--primary) / 0.14)"
      : props.$kind === "error"
        ? "hsl(var(--destructive) / 0.12)"
        : "hsl(36 100% 90%)"};
`;

const ThemeWorkbenchRunningBar = styled.div`
  min-height: 44px;
  border: 1px solid hsl(var(--border));
  border-radius: 11px;
  background: hsl(var(--background));
  box-shadow: 0 4px 14px hsl(var(--foreground) / 0.04);
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 7px 10px;
`;

const ThemeWorkbenchRunningIcon = styled.span`
  color: hsl(var(--primary));
  display: inline-flex;
  flex-shrink: 0;
`;

const ThemeWorkbenchRunningSub = styled.span`
  flex: 1;
  min-width: 0;
  font-size: 12px;
  color: hsl(var(--muted-foreground));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
`;

const ThemeWorkbenchRunningMain = styled.span`
  color: hsl(var(--primary));
  font-weight: 600;
  margin-right: 2px;
  font-size: 14px;
`;

const ThemeWorkbenchStopButton = styled.button`
  width: 24px;
  height: 24px;
  border-radius: 999px;
  border: 1px solid hsl(var(--border));
  background: hsl(var(--muted) / 0.28);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: hsl(var(--muted-foreground));
  flex-shrink: 0;
  position: relative;

  &:hover {
    color: hsl(var(--destructive));
    border-color: hsl(var(--destructive) / 0.5);
    background: hsl(var(--destructive) / 0.06);
  }
`;

const ThemeWorkbenchStopGlyph = styled.span`
  width: 12px;
  height: 12px;
  border: 1.5px solid currentColor;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;

  &::after {
    content: "";
    width: 3px;
    height: 3px;
    border-radius: 999px;
    background: currentColor;
  }
`;

interface HintRouteItem {
  hint: string;
  provider: string;
  model: string;
}

interface ThemeWorkbenchQuickAction {
  id: string;
  label: string;
  prompt: string;
}

export interface InputbarToolStates {
  webSearch: boolean;
  thinking: boolean;
}

export interface ThemeWorkbenchGateState {
  key: string;
  title: string;
  status: "running" | "waiting" | "idle";
  description: string;
}

interface ThemeWorkbenchWorkflowStep {
  id: string;
  title: string;
  status: StepStatus;
}

const DEFAULT_INPUTBAR_TOOL_STATES: InputbarToolStates = {
  webSearch: false,
  thinking: false,
};

function resolveThemeWorkbenchQuickActions(
  gateKey?: string,
): ThemeWorkbenchQuickAction[] {
  switch (gateKey) {
    case "topic_select":
      return [
        {
          id: "topic-options",
          label: "生成 3 个选题",
          prompt: "请给我 3 个可执行选题方向，并说明目标读者与传播价值。",
        },
        {
          id: "topic-choose-b",
          label: "采纳 B 方向",
          prompt: "我采纳 B 方向，请继续推进主稿与配图编排。",
        },
      ];
    case "write_mode":
      return [
        {
          id: "write-fast",
          label: "快速模式出稿",
          prompt: "请按快速模式生成可发布主稿，并标注可优化段落。",
        },
        {
          id: "write-coach",
          label: "教练模式引导",
          prompt: "请按教练模式逐步提问我，帮助补充真实案例后再成稿。",
        },
      ];
    case "publish_confirm":
      return [
        {
          id: "publish-checklist",
          label: "发布前检查",
          prompt: "请给我发布前检查清单，包含标题、封面、平台合规与风险项。",
        },
        {
          id: "publish-adapt",
          label: "双平台适配",
          prompt: "请将主稿适配为公众号和小红书两个版本，并输出差异点。",
        },
      ];
    default:
      return [
        {
          id: "next-step",
          label: "继续编排",
          prompt: "请继续按照当前编排推进，并在关键闸门前向我确认。",
        },
      ];
  }
}

interface InputbarProps {
  input: string;
  setInput: (value: string) => void;
  onSend: (
    images?: MessageImage[],
    webSearch?: boolean,
    thinking?: boolean,
    textOverride?: string,
    executionStrategy?: "react" | "code_orchestrated" | "auto",
  ) => void;
  /** 停止生成回调 */
  onStop?: () => void;
  isLoading: boolean;
  disabled?: boolean;
  onClearMessages?: () => void;
  /** 切换画布显示 */
  onToggleCanvas?: () => void;
  /** 画布是否打开 */
  isCanvasOpen?: boolean;
  /** 任务文件列表 */
  taskFiles?: TaskFile[];
  /** 选中的文件 ID */
  selectedFileId?: string;
  /** 任务文件面板是否展开 */
  taskFilesExpanded?: boolean;
  /** 切换任务文件面板 */
  onToggleTaskFiles?: () => void;
  /** 文件点击回调 */
  onTaskFileClick?: (file: TaskFile) => void;
  /** 角色列表（用于 @ 引用） */
  characters?: Character[];
  /** 技能列表（用于 @ 引用） */
  skills?: Skill[];
  /** 选择角色回调 */
  onSelectCharacter?: (character: Character) => void;
  /** 跳转到设置页安装技能 */
  onNavigateToSettings?: () => void;
  providerType?: string;
  setProviderType?: (type: string) => void;
  model?: string;
  setModel?: (model: string) => void;
  executionStrategy?: "react" | "code_orchestrated" | "auto";
  setExecutionStrategy?: (
    strategy: "react" | "code_orchestrated" | "auto",
  ) => void;
  toolStates?: Partial<InputbarToolStates>;
  onToolStatesChange?: (states: InputbarToolStates) => void;
  activeTheme?: string;
  onManageProviders?: () => void;
  variant?: "default" | "theme_workbench";
  themeWorkbenchGate?: ThemeWorkbenchGateState | null;
  workflowSteps?: ThemeWorkbenchWorkflowStep[];
  themeWorkbenchRunState?: "idle" | "auto_running" | "await_user_decision";
}

export const Inputbar: React.FC<InputbarProps> = ({
  input,
  setInput,
  onSend,
  onStop,
  isLoading,
  disabled,
  onClearMessages,
  onToggleCanvas,
  isCanvasOpen = false,
  taskFiles = [],
  selectedFileId,
  taskFilesExpanded = false,
  onToggleTaskFiles,
  onTaskFileClick,
  characters = [],
  skills = [],
  onSelectCharacter,
  onNavigateToSettings,
  providerType,
  setProviderType,
  model,
  setModel,
  executionStrategy,
  setExecutionStrategy,
  toolStates,
  onToolStatesChange,
  activeTheme,
  onManageProviders,
  variant = "default",
  themeWorkbenchGate,
  workflowSteps = [],
  themeWorkbenchRunState,
}) => {
  const [localActiveTools, setLocalActiveTools] = useState<
    Record<string, boolean>
  >({});
  const [localToolStates, setLocalToolStates] = useState<InputbarToolStates>(
    DEFAULT_INPUTBAR_TOOL_STATES,
  );
  const [pendingImages, setPendingImages] = useState<MessageImage[]>([]);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [themeWorkbenchQueueCollapsed, setThemeWorkbenchQueueCollapsed] =
    useState(false);
  const { activeSkill, setActiveSkill, clearActiveSkill } = useActiveSkill();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Hint 路由
  const [showHintPopup, setShowHintPopup] = useState(false);
  const [hintRoutes, setHintRoutes] = useState<HintRouteItem[]>([]);
  const [hintIndex, setHintIndex] = useState(0);

  const webSearchEnabled =
    toolStates?.webSearch ?? localToolStates.webSearch;
  const thinkingEnabled = toolStates?.thinking ?? localToolStates.thinking;
  const isThemeWorkbenchVariant = variant === "theme_workbench";
  const themeWorkbenchQuickActions = useMemo(
    () =>
      isThemeWorkbenchVariant
        ? resolveThemeWorkbenchQuickActions(themeWorkbenchGate?.key)
        : [],
    [isThemeWorkbenchVariant, themeWorkbenchGate?.key],
  );
  const themeWorkbenchQueueItems = useMemo(() => {
    if (!isThemeWorkbenchVariant) {
      return [];
    }
    const visibleSteps = workflowSteps
      .filter((step) => step.status !== "completed" && step.status !== "skipped")
      .slice(0, 3);
    if (visibleSteps.length > 0) {
      return visibleSteps;
    }
    if (themeWorkbenchGate) {
      return [
        {
          id: `gate-${themeWorkbenchGate.key}`,
          title: themeWorkbenchGate.title,
          status:
            themeWorkbenchGate.status === "waiting"
              ? ("pending" as StepStatus)
              : ("active" as StepStatus),
        },
      ];
    }
    return [];
  }, [isThemeWorkbenchVariant, themeWorkbenchGate, workflowSteps]);

  const activeTools = useMemo<Record<string, boolean>>(
    () => ({
      ...localActiveTools,
      web_search: webSearchEnabled,
      thinking: thinkingEnabled,
    }),
    [localActiveTools, thinkingEnabled, webSearchEnabled],
  );

  const updateToolStates = useCallback(
    (next: InputbarToolStates) => {
      setLocalToolStates((prev) => ({
        webSearch: toolStates?.webSearch ?? next.webSearch ?? prev.webSearch,
        thinking: toolStates?.thinking ?? next.thinking ?? prev.thinking,
      }));
      onToolStatesChange?.(next);
      return next;
    },
    [onToolStatesChange, toolStates?.thinking, toolStates?.webSearch],
  );

  useEffect(() => {
    safeInvoke<HintRouteItem[]>("get_hint_routes")
      .then((routes) => {
        if (routes?.length > 0) {
          setHintRoutes(routes);
        }
      })
      .catch(() => {});
  }, []);

  // 监听输入变化，触发 hint 弹出
  const handleSetInput = useCallback(
    (value: string) => {
      setInput(value);
      if (hintRoutes.length > 0 && value === "[") {
        setShowHintPopup(true);
        setHintIndex(0);
      } else if (!value.startsWith("[") || value.includes("]")) {
        setShowHintPopup(false);
      }
    },
    [hintRoutes.length, setInput],
  );

  const handleHintSelect = useCallback(
    (hint: string) => {
      setInput(`[${hint}] `);
      setShowHintPopup(false);
      textareaRef.current?.focus();
    },
    [setInput],
  );

  const handleHintKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const nativeEvent = e.nativeEvent as KeyboardEvent & {
        isComposing?: boolean;
      };
      if (
        nativeEvent.isComposing ||
        nativeEvent.key === "Process" ||
        nativeEvent.keyCode === 229
      ) {
        return;
      }
      if (!showHintPopup || hintRoutes.length === 0) return;
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setHintIndex((i) => (i + 1) % hintRoutes.length);
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setHintIndex((i) => (i - 1 + hintRoutes.length) % hintRoutes.length);
      } else if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        handleHintSelect(hintRoutes[hintIndex].hint);
      } else if (e.key === "Escape") {
        setShowHintPopup(false);
      }
    },
    [handleHintSelect, hintIndex, hintRoutes, showHintPopup],
  );

  const handleToolClick = useCallback(
    (tool: string) => {
      switch (tool) {
        case "thinking": {
          const nextThinking = !thinkingEnabled;
          updateToolStates({
            webSearch: webSearchEnabled,
            thinking: nextThinking,
          });
          toast.info(`深度思考${nextThinking ? "已开启" : "已关闭"}`);
          break;
        }
        case "web_search": {
          const nextWebSearch = !webSearchEnabled;
          updateToolStates({
            webSearch: nextWebSearch,
            thinking: thinkingEnabled,
          });
          toast.info(`联网搜索${nextWebSearch ? "已开启" : "已关闭"}`);
          break;
        }
        case "execution_strategy":
          if (setExecutionStrategy) {
            const strategyOrder: Array<
              "react" | "code_orchestrated" | "auto"
            > = ["react", "code_orchestrated", "auto"];
            const currentIndex = strategyOrder.indexOf(
              executionStrategy || "react",
            );
            const nextStrategy =
              strategyOrder[(currentIndex + 1) % strategyOrder.length];
            setExecutionStrategy(nextStrategy);
            toast.info(
              nextStrategy === "react"
                ? "执行模式：ReAct"
                : nextStrategy === "code_orchestrated"
                  ? "执行模式：Plan"
                  : "执行模式：Auto",
            );
            break;
          }
          setLocalActiveTools((prev) => {
            const enabled = !prev["execution_strategy"];
            toast.info(`Plan 模式${enabled ? "已开启" : "已关闭"}`);
            return { ...prev, execution_strategy: enabled };
          });
          break;
        case "clear":
          setInput("");
          setPendingImages([]);
          toast.success("已清除输入");
          break;
        case "new_topic":
          onClearMessages?.();
          setInput("");
          setPendingImages([]);
          break;
        case "attach":
          fileInputRef.current?.click();
          break;
        case "quick_action":
        case "translate":
          toast.info("翻译功能开发中...");
          break;
        case "fullscreen":
          setIsFullscreen((prev) => !prev);
          toast.info(isFullscreen ? "已退出全屏" : "已进入全屏编辑");
          break;
        case "canvas":
          onToggleCanvas?.();
          break;
        default:
          break;
      }
    },
    [
      executionStrategy,
      thinkingEnabled,
      onClearMessages,
      onToggleCanvas,
      setExecutionStrategy,
      setInput,
      updateToolStates,
      webSearchEnabled,
      isFullscreen,
    ],
  );

  const handleFileSelect = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files;
      if (!files || files.length === 0) return;

      Array.from(files).forEach((file) => {
        if (file.type.startsWith("image/")) {
          const reader = new FileReader();
          reader.onload = (event) => {
            const base64 = event.target?.result as string;
            const base64Data = base64.split(",")[1];
            setPendingImages((prev) => [
              ...prev,
              {
                data: base64Data,
                mediaType: file.type,
              },
            ]);
            toast.success(`已添加图片: ${file.name}`);
          };
          reader.readAsDataURL(file);
        } else {
          toast.info(`暂不支持该文件类型: ${file.type}`);
        }
      });

      e.target.value = "";
    },
    [],
  );

  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    const items = e.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
      if (item.type.startsWith("image/")) {
        e.preventDefault();
        const file = item.getAsFile();
        if (file) {
          const reader = new FileReader();
          reader.onload = (event) => {
            const base64 = event.target?.result as string;
            const base64Data = base64.split(",")[1];
            setPendingImages((prev) => [
              ...prev,
              {
                data: base64Data,
                mediaType: item.type,
              },
            ]);
            toast.success("已粘贴图片");
          };
          reader.readAsDataURL(file);
        }
        break;
      }
    }
  }, []);

  // 文件拖拽处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const files = e.dataTransfer.files;
    if (!files || files.length === 0) return;

    Array.from(files).forEach((file) => {
      if (file.type.startsWith("image/")) {
        const reader = new FileReader();
        reader.onload = (event) => {
          const base64 = event.target?.result as string;
          const base64Data = base64.split(",")[1];
          setPendingImages((prev) => [
            ...prev,
            {
              data: base64Data,
              mediaType: file.type,
            },
          ]);
          toast.success(`已添加图片: ${file.name}`);
        };
        reader.readAsDataURL(file);
      } else {
        toast.info(`暂不支持该文件类型: ${file.type}`);
      }
    });
  }, []);

  const handleRemoveImage = useCallback((index: number) => {
    setPendingImages((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleSend = useCallback(() => {
    if (!input.trim() && pendingImages.length === 0) return;
    const webSearch = webSearchEnabled;
    const thinking = thinkingEnabled;
    let strategy =
      executionStrategy ||
      (activeTools["execution_strategy"] ? "code_orchestrated" : "react");

    if (webSearch && strategy !== "react") {
      strategy = "react";
    }

    // 如果有 activeSkill，拼接 /skill.key 前缀
    let textOverride: string | undefined;
    if (activeSkill) {
      textOverride = `/${activeSkill.key} ${input}`.trim();
    } else if (
      activeTheme === "social-media" &&
      input.trim() &&
      !input.trimStart().startsWith("/")
    ) {
      textOverride = `/${SOCIAL_ARTICLE_SKILL_KEY} ${input}`.trim();
    }

    onSend(
      pendingImages.length > 0 ? pendingImages : undefined,
      webSearch,
      thinking,
      textOverride,
      strategy,
    );
    setPendingImages([]);
    clearActiveSkill();
  }, [
    activeSkill,
    activeTools,
    clearActiveSkill,
    executionStrategy,
    input,
    activeTheme,
    onSend,
    pendingImages,
    thinkingEnabled,
    webSearchEnabled,
  ]);

  const handleToggleTaskFiles = useCallback(() => {
    onToggleTaskFiles?.();
  }, [onToggleTaskFiles]);

  const resolvedExecutionStrategy = executionStrategy || "react";
  const executionStrategyLabel =
    resolvedExecutionStrategy === "auto"
      ? "Auto"
      : resolvedExecutionStrategy === "code_orchestrated"
        ? "Plan"
        : "ReAct";

  const inputAdapter = useMemo(
    () =>
      createAgentInputAdapter({
        text: input,
        setText: handleSetInput,
        isSending: isLoading,
        disabled,
        providerType: providerType || "",
        model: model || "",
        setProviderType: setProviderType || NOOP_SET_PROVIDER_TYPE,
        setModel: setModel || NOOP_SET_MODEL,
        send: () => handleSend(),
        stop: onStop,
        attachments: pendingImages,
        showExecutionStrategy: Boolean(setExecutionStrategy),
      }),
    [
      disabled,
      handleSend,
      handleSetInput,
      input,
      isLoading,
      model,
      onStop,
      pendingImages,
      providerType,
      setExecutionStrategy,
      setModel,
      setProviderType,
    ],
  );

  const shouldRenderModelSelector = Boolean(
    !isThemeWorkbenchVariant &&
      providerType &&
      setProviderType &&
      model &&
      setModel,
  );
  const topExtra = activeSkill ? (
    <SkillBadge skill={activeSkill} onClear={clearActiveSkill} />
  ) : undefined;

  const themeWorkbenchGateStrip =
    isThemeWorkbenchVariant &&
    themeWorkbenchGate &&
    themeWorkbenchGate.status !== "idle" ? (
      <ThemeWorkbenchGateStrip>
        <ThemeWorkbenchGateMeta>
          <ThemeWorkbenchGateIcon>
            <Sparkles size={13} />
          </ThemeWorkbenchGateIcon>
          <ThemeWorkbenchGateTitle>{themeWorkbenchGate.title}</ThemeWorkbenchGateTitle>
          <ThemeWorkbenchGateStatus $status={themeWorkbenchGate.status}>
            {themeWorkbenchGate.status === "waiting"
              ? "等待决策"
              : themeWorkbenchGate.status === "running"
                ? "自动执行中"
                : "待启动"}
          </ThemeWorkbenchGateStatus>
        </ThemeWorkbenchGateMeta>
        {themeWorkbenchQuickActions.length > 0 ? (
          <ThemeWorkbenchQuickActions>
            {themeWorkbenchQuickActions.map((action) => (
              <ThemeWorkbenchQuickButton
                key={action.id}
                type="button"
                onClick={() => {
                  inputAdapter.actions.setText(action.prompt);
                }}
              >
                {action.label}
              </ThemeWorkbenchQuickButton>
            ))}
          </ThemeWorkbenchQuickActions>
        ) : null}
      </ThemeWorkbenchGateStrip>
    ) : null;

  const renderThemeWorkbenchGeneratingPanel = isThemeWorkbenchVariant
    ? themeWorkbenchRunState
      ? themeWorkbenchRunState === "auto_running"
      : inputAdapter.state.isSending
    : false;

  return (
    <div
      onDragOver={handleDragOver}
      onDrop={handleDrop}
      onKeyDown={handleHintKeyDown}
      className={
        isFullscreen ? "fixed inset-0 z-50 bg-background p-4 flex flex-col" : ""
      }
      style={{ position: "relative" }}
    >
      {/* Hint 路由弹出框 */}
      {showHintPopup && hintRoutes.length > 0 && (
        <HintPopup>
          {hintRoutes.map((route, i) => (
            <HintItem
              key={route.hint}
              $active={i === hintIndex}
              onClick={() => handleHintSelect(route.hint)}
            >
              <HintLabel>[{route.hint}]</HintLabel>
              <HintModel>{route.provider} / {route.model}</HintModel>
            </HintItem>
          ))}
        </HintPopup>
      )}
      {/* 任务文件区域 - 在输入框上方 */}
      {taskFiles.length > 0 && (
        <TaskFilesArea>
          {/* 按钮和面板的包装容器 */}
          <TaskFilesWrapper>
            {/* 任务文件面板 */}
            <TaskFileList
              files={taskFiles}
              selectedFileId={selectedFileId}
              onFileClick={onTaskFileClick}
              expanded={taskFilesExpanded}
              onExpandedChange={(expanded) => {
                if (expanded !== taskFilesExpanded) {
                  onToggleTaskFiles?.();
                }
              }}
            />
            {/* 任务文件按钮 */}
            <TaskFilesButton
              $hasFiles={taskFiles.length > 0}
              $expanded={taskFilesExpanded}
              onClick={handleToggleTaskFiles}
              data-task-files-trigger
            >
              <FolderOpen size={14} />
              任务文件
              <FileCount>({taskFiles.length})</FileCount>
              <ChevronIcon $expanded={taskFilesExpanded}>
                <ChevronUp size={14} />
              </ChevronIcon>
            </TaskFilesButton>
          </TaskFilesWrapper>
        </TaskFilesArea>
      )}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        multiple
        style={{ display: "none" }}
        onChange={handleFileSelect}
      />
      {renderThemeWorkbenchGeneratingPanel ? (
        <ThemeWorkbenchGeneratingWrap>
          <ThemeWorkbenchTaskCard>
            <ThemeWorkbenchTaskHead>
              <ThemeWorkbenchTaskHeadButton
                type="button"
                onClick={() => setThemeWorkbenchQueueCollapsed((prev) => !prev)}
                aria-label={
                  themeWorkbenchQueueCollapsed
                    ? "展开待办列表"
                    : "折叠待办列表"
                }
              >
                <span>当前待办</span>
                <ThemeWorkbenchTaskHeadChevron
                  $collapsed={themeWorkbenchQueueCollapsed}
                >
                  <ChevronDown size={14} />
                </ThemeWorkbenchTaskHeadChevron>
              </ThemeWorkbenchTaskHeadButton>
            </ThemeWorkbenchTaskHead>
            {!themeWorkbenchQueueCollapsed ? (
              <ThemeWorkbenchTaskList>
                {themeWorkbenchQueueItems.length === 0 ? (
                  <ThemeWorkbenchTaskRow>
                    <ThemeWorkbenchTaskIcon $kind="active">
                      <Loader2 size={14} className="animate-spin" />
                    </ThemeWorkbenchTaskIcon>
                    <ThemeWorkbenchTaskText>正在编排任务节点...</ThemeWorkbenchTaskText>
                    <ThemeWorkbenchTaskStatus $kind="active">
                      进行中
                    </ThemeWorkbenchTaskStatus>
                  </ThemeWorkbenchTaskRow>
                ) : (
                  themeWorkbenchQueueItems.map((item) => {
                    const statusKind =
                      item.status === "active"
                        ? "active"
                        : item.status === "error"
                          ? "error"
                          : "pending";
                    return (
                      <ThemeWorkbenchTaskRow key={item.id}>
                        <ThemeWorkbenchTaskIcon $kind={statusKind}>
                          {statusKind === "active" ? (
                            <Loader2 size={14} className="animate-spin" />
                          ) : statusKind === "error" ? (
                            <AlertCircle size={14} />
                          ) : (
                            <Clock3 size={14} />
                          )}
                        </ThemeWorkbenchTaskIcon>
                        <ThemeWorkbenchTaskText>{item.title}</ThemeWorkbenchTaskText>
                        <ThemeWorkbenchTaskStatus $kind={statusKind}>
                          {statusKind === "active"
                            ? "进行中"
                            : statusKind === "error"
                              ? "异常"
                              : "待处理"}
                        </ThemeWorkbenchTaskStatus>
                      </ThemeWorkbenchTaskRow>
                    );
                  })
                )}
              </ThemeWorkbenchTaskList>
            ) : null}
          </ThemeWorkbenchTaskCard>
          <ThemeWorkbenchRunningBar>
            <ThemeWorkbenchRunningIcon>
              <Sparkles size={13} />
            </ThemeWorkbenchRunningIcon>
            <ThemeWorkbenchRunningMain>正在生成中 • • •</ThemeWorkbenchRunningMain>
            <ThemeWorkbenchRunningSub>切换项目或关闭网页将中断任务</ThemeWorkbenchRunningSub>
            <ThemeWorkbenchStopButton
              type="button"
              data-testid="theme-workbench-stop"
              onClick={() => inputAdapter.actions.stop?.()}
              aria-label="停止生成"
            >
              <ThemeWorkbenchStopGlyph />
            </ThemeWorkbenchStopButton>
          </ThemeWorkbenchRunningBar>
        </ThemeWorkbenchGeneratingWrap>
      ) : (
        <>
          {themeWorkbenchGateStrip}
          <CharacterMention
            characters={characters}
            skills={skills}
            inputRef={textareaRef}
            value={input}
            onChange={inputAdapter.actions.setText}
            onSelectCharacter={onSelectCharacter}
            onSelectSkill={setActiveSkill}
            onNavigateToSettings={onNavigateToSettings}
          />
          <InputbarCore
            textareaRef={textareaRef}
            text={inputAdapter.state.text}
            setText={inputAdapter.actions.setText}
            onSend={handleSend}
            onStop={inputAdapter.actions.stop}
            isLoading={inputAdapter.state.isSending}
            disabled={inputAdapter.state.disabled}
            onToolClick={handleToolClick}
            activeTools={activeTools}
            executionStrategy={executionStrategy}
            showExecutionStrategy={false}
            pendingImages={
              (inputAdapter.state.attachments as MessageImage[] | undefined) ||
              pendingImages
            }
            onRemoveImage={handleRemoveImage}
            onPaste={handlePaste}
            isFullscreen={isFullscreen}
            isCanvasOpen={isCanvasOpen}
            placeholder={
              isThemeWorkbenchVariant
                ? themeWorkbenchGate?.status === "waiting"
                  ? "说说你的选择，剩下的交给我"
                  : "试着输入任何指令，剩下的交给我"
                : undefined
            }
            toolMode={isThemeWorkbenchVariant ? "attach-only" : "default"}
            showTranslate={!isThemeWorkbenchVariant}
            showDragHandle={!isThemeWorkbenchVariant}
            visualVariant={isThemeWorkbenchVariant ? "floating" : "default"}
            topExtra={topExtra}
            leftExtra={
              !isFullscreen && !isThemeWorkbenchVariant ? (
                <div className="flex items-center gap-2">
                  {shouldRenderModelSelector && inputAdapter.model ? (
                    <ChatModelSelector
                      providerType={inputAdapter.model.providerType}
                      setProviderType={inputAdapter.actions.setProviderType || NOOP_SET_PROVIDER_TYPE}
                      model={inputAdapter.model.model}
                      setModel={inputAdapter.actions.setModel || NOOP_SET_MODEL}
                      activeTheme={activeTheme}
                      compactTrigger
                      popoverSide="top"
                      onManageProviders={onManageProviders}
                    />
                  ) : null}
                </div>
              ) : undefined
            }
            rightExtra={
              !isFullscreen && !isThemeWorkbenchVariant && setExecutionStrategy ? (
                <Select
                  value={resolvedExecutionStrategy}
                  onValueChange={(value) =>
                    setExecutionStrategy(
                      value as "react" | "code_orchestrated" | "auto",
                    )
                  }
                >
                  <SelectTrigger className="h-8 text-xs bg-background border shadow-sm min-w-[116px] px-2">
                    <div className="flex items-center gap-1.5">
                      <Code2 className="w-3.5 h-3.5 text-muted-foreground" />
                      <span className="whitespace-nowrap">
                        {executionStrategyLabel}
                      </span>
                    </div>
                  </SelectTrigger>
                  <SelectContent side="top" className="p-1 w-[176px]">
                    <SelectItem value="react">
                      <div className="flex items-center gap-2 whitespace-nowrap">
                        <Code2 className="w-3.5 h-3.5" />
                        ReAct
                      </div>
                    </SelectItem>
                    <SelectItem value="code_orchestrated">
                      <div className="flex items-center gap-2 whitespace-nowrap">
                        <Code2 className="w-3.5 h-3.5" />
                        Plan
                      </div>
                    </SelectItem>
                    <SelectItem value="auto">
                      <div className="flex items-center gap-2 whitespace-nowrap">
                        <Code2 className="w-3.5 h-3.5" />
                        Auto
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              ) : undefined
            }
          />
        </>
      )}
    </div>
  );
};
