import { useCallback } from "react";
import type { Skill } from "@/lib/api/skills";
import type { MessageImage } from "../../../types";
import type { BuiltinInputCommand } from "../components/builtinCommands";

const SOCIAL_ARTICLE_SKILL_KEY = "social_post_with_cover";

interface UseInputbarSendParams {
  input: string;
  pendingImages: MessageImage[];
  webSearchEnabled: boolean;
  thinkingEnabled: boolean;
  executionStrategy?: "react" | "code_orchestrated" | "auto";
  activeTools: Record<string, boolean>;
  activeSkill: Skill | null;
  activeBuiltinCommand: BuiltinInputCommand | null;
  activeTheme?: string;
  onSend: (
    images?: MessageImage[],
    webSearch?: boolean,
    thinking?: boolean,
    textOverride?: string,
    executionStrategy?: "react" | "code_orchestrated" | "auto",
  ) => void | Promise<boolean> | boolean;
  clearPendingImages: () => void;
  clearActiveSkill: () => void;
  clearActiveBuiltinCommand: () => void;
}

export function useInputbarSend({
  input,
  pendingImages,
  webSearchEnabled,
  thinkingEnabled,
  executionStrategy,
  activeTools,
  activeSkill,
  activeBuiltinCommand,
  activeTheme,
  onSend,
  clearPendingImages,
  clearActiveSkill,
  clearActiveBuiltinCommand,
}: UseInputbarSendParams) {
  return useCallback(async () => {
    if (!input.trim() && pendingImages.length === 0) {
      return;
    }

    const webSearch = webSearchEnabled;
    const thinking = thinkingEnabled;
    let strategy =
      executionStrategy ||
      (activeTools["execution_strategy"] ? "code_orchestrated" : "react");

    if (webSearch && strategy !== "react") {
      strategy = "react";
    }

    let textOverride: string | undefined;
    if (activeBuiltinCommand) {
      textOverride = `${activeBuiltinCommand.commandPrefix} ${input}`.trim();
    } else if (activeSkill) {
      textOverride = `/${activeSkill.key} ${input}`.trim();
    } else if (
      activeTheme === "social-media" &&
      input.trim() &&
      !input.trimStart().startsWith("/")
    ) {
      textOverride = `/${SOCIAL_ARTICLE_SKILL_KEY} ${input}`.trim();
    }

    try {
      const result = await onSend(
        pendingImages.length > 0 ? pendingImages : undefined,
        webSearch,
        thinking,
        textOverride,
        strategy,
      );
      if (result === false) {
        return;
      }
      clearPendingImages();
      clearActiveSkill();
      clearActiveBuiltinCommand();
    } catch {
      // 发送失败时保留图片与技能，交由上层 toast / 恢复逻辑处理。
    }
  }, [
    activeBuiltinCommand,
    activeSkill,
    activeTheme,
    activeTools,
    clearActiveBuiltinCommand,
    clearActiveSkill,
    clearPendingImages,
    executionStrategy,
    input,
    onSend,
    pendingImages,
    thinkingEnabled,
    webSearchEnabled,
  ]);
}
