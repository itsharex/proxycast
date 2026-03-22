/**
 * 角色与技能引用组件
 *
 * 在输入框中检测 @ 符号，显示角色和技能列表供选择
 */

import React, {
  Suspense,
  lazy,
  useState,
  useEffect,
  useMemo,
  useRef,
  useCallback,
} from "react";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { Character } from "@/lib/api/memory";
import type { Skill } from "@/lib/api/skills";
import { toast } from "sonner";
import { scheduleIdleModulePreload } from "./scheduleIdleModulePreload";
import {
  filterBuiltinCommands,
  type BuiltinInputCommand,
} from "./builtinCommands";

const preloadCharacterMentionPanel = () => import("./CharacterMentionPanel");

const CharacterMentionPanel = lazy(async () => {
  const module = await preloadCharacterMentionPanel();
  return { default: module.CharacterMentionPanel };
});

interface CharacterMentionProps {
  /** 角色列表 */
  characters: Character[];
  /** 技能列表 */
  skills?: Skill[];
  /** 输入框 ref */
  inputRef: React.RefObject<HTMLTextAreaElement>;
  /** 当前输入值 */
  value: string;
  /** 输入值变更回调 */
  onChange: (value: string) => void;
  /** 选择角色回调 */
  onSelectCharacter?: (character: Character) => void;
  /** 选择已安装技能回调 */
  onSelectSkill?: (skill: Skill) => void;
  /** 选择内建命令回调 */
  onSelectBuiltinCommand?: (command: BuiltinInputCommand) => void;
  /** 跳转到设置页安装技能 */
  onNavigateToSettings?: () => void;
}

export function CharacterMention({
  characters,
  skills = [],
  inputRef,
  value,
  onChange,
  onSelectCharacter,
  onSelectSkill,
  onSelectBuiltinCommand,
  onNavigateToSettings,
}: CharacterMentionProps) {
  const [showMentions, setShowMentions] = useState(false);
  const [mentionQuery, setMentionQuery] = useState("");
  const [panelAnchor, setPanelAnchor] = useState({
    top: 0,
    left: 0,
    width: 320,
  });
  const popoverRef = useRef<HTMLDivElement>(null);
  const commandRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    return scheduleIdleModulePreload(() => {
      void preloadCharacterMentionPanel();
    });
  }, []);

  const filteredBuiltinCommands = useMemo(
    () => filterBuiltinCommands(mentionQuery),
    [mentionQuery],
  );

  // 过滤角色列表
  const filteredCharacters = useMemo(() => {
    if (!mentionQuery) return characters;
    const query = mentionQuery.toLowerCase();
    return characters.filter(
      (char) =>
        char.name.toLowerCase().includes(query) ||
        char.description?.toLowerCase().includes(query),
    );
  }, [characters, mentionQuery]);

  // 过滤已安装技能
  const installedSkills = useMemo(() => {
    const installed = skills.filter((s) => s.installed);
    if (!mentionQuery) return installed;
    const query = mentionQuery.toLowerCase();
    return installed.filter(
      (s) =>
        s.name.toLowerCase().includes(query) ||
        s.key.toLowerCase().includes(query) ||
        s.description?.toLowerCase().includes(query),
    );
  }, [skills, mentionQuery]);

  // 过滤未安装技能
  const availableSkills = useMemo(() => {
    const available = skills.filter((s) => !s.installed);
    if (!mentionQuery) return available;
    const query = mentionQuery.toLowerCase();
    return available.filter(
      (s) =>
        s.name.toLowerCase().includes(query) ||
        s.key.toLowerCase().includes(query) ||
        s.description?.toLowerCase().includes(query),
    );
  }, [skills, mentionQuery]);

  const updateMentionState = useCallback(() => {
    const textarea = inputRef.current;
    if (!textarea) {
      setShowMentions(false);
      return;
    }

    const cursorPos = textarea.selectionStart ?? textarea.value.length;
    const textBeforeCursor = textarea.value.slice(0, cursorPos);
    const lastAtIndex = textBeforeCursor.lastIndexOf("@");

    if (lastAtIndex !== -1) {
      const textAfterAt = textBeforeCursor.slice(lastAtIndex + 1);
      if (!textAfterAt.includes(" ") && !textAfterAt.includes("\n")) {
        setMentionQuery(textAfterAt);
        setShowMentions(true);

        const rect = textarea.getBoundingClientRect();
        const viewportWidth =
          window.innerWidth || document.documentElement.clientWidth || 1280;
        const panelWidth = Math.min(Math.max(rect.width - 24, 320), 420);
        const centerLeft = rect.left + rect.width / 2;
        const safeHalfWidth = panelWidth / 2 + 16;
        const left = Math.min(
          Math.max(centerLeft, safeHalfWidth),
          viewportWidth - safeHalfWidth,
        );
        setPanelAnchor({
          top: rect.top,
          left,
          width: panelWidth,
        });
        return;
      }
    }

    setShowMentions(false);
  }, [inputRef]);

  // 检测 @ 符号
  useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea) return;
    textarea.addEventListener("input", updateMentionState);
    textarea.addEventListener("click", updateMentionState);
    textarea.addEventListener("keyup", updateMentionState);

    return () => {
      textarea.removeEventListener("input", updateMentionState);
      textarea.removeEventListener("click", updateMentionState);
      textarea.removeEventListener("keyup", updateMentionState);
    };
  }, [inputRef, updateMentionState]);

  useEffect(() => {
    updateMentionState();
  }, [updateMentionState, value]);

  useEffect(() => {
    if (!showMentions) {
      return;
    }

    const handleReposition = () => {
      updateMentionState();
    };

    window.addEventListener("resize", handleReposition);
    window.addEventListener("scroll", handleReposition, true);
    return () => {
      window.removeEventListener("resize", handleReposition);
      window.removeEventListener("scroll", handleReposition, true);
    };
  }, [showMentions, updateMentionState]);

  // 插入角色引用
  const handleSelectCharacter = (character: Character) => {
    const textarea = inputRef.current;
    if (!textarea) return;

    const cursorPos = textarea.selectionStart;
    const textBeforeCursor = value.slice(0, cursorPos);
    const textAfterCursor = value.slice(cursorPos);
    const lastAtIndex = textBeforeCursor.lastIndexOf("@");

    // 替换 @ 和后面的查询文本为角色名
    const newValue =
      value.slice(0, lastAtIndex) + `@${character.name} ` + textAfterCursor;

    onChange(newValue);
    setShowMentions(false);

    // 通知父组件
    onSelectCharacter?.(character);

    // 恢复焦点并设置光标位置
    setTimeout(() => {
      textarea.focus();
      const newCursorPos = lastAtIndex + character.name.length + 2; // @ + 名字 + 空格
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  // 选择已安装技能 → 通知父组件，清除 @ 查询文本
  const handleSelectInstalledSkill = (skill: Skill) => {
    const textarea = inputRef.current;
    if (!textarea) return;

    const cursorPos = textarea.selectionStart;
    const textBeforeCursor = value.slice(0, cursorPos);
    const textAfterCursor = value.slice(cursorPos);
    const lastAtIndex = textBeforeCursor.lastIndexOf("@");

    // Inputbar 场景：由父组件接管 activeSkill（显示 SkillBadge）
    if (onSelectSkill) {
      const newValue = value.slice(0, lastAtIndex) + textAfterCursor;
      onChange(newValue.trimEnd() === "" ? "" : newValue);
      setShowMentions(false);
      onSelectSkill(skill);

      setTimeout(() => {
        textarea.focus();
        const newCursorPos = Math.max(0, lastAtIndex);
        textarea.setSelectionRange(newCursorPos, newCursorPos);
      }, 0);
      return;
    }

    // 通用场景（例如 EmptyState）：直接回填为 /skillKey，保证可见且可发送
    const newValue =
      value.slice(0, lastAtIndex) + `/${skill.key} ` + textAfterCursor;
    onChange(newValue);
    setShowMentions(false);

    setTimeout(() => {
      textarea.focus();
      const newCursorPos = lastAtIndex + skill.key.length + 2;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  // 选择未安装技能 → toast 提示
  const handleSelectAvailableSkill = (skill: Skill) => {
    setShowMentions(false);

    toast.info(`技能「${skill.name}」尚未安装`, {
      action: onNavigateToSettings
        ? {
            label: "去安装",
            onClick: onNavigateToSettings,
          }
        : undefined,
    });
  };

  const handleSelectBuiltinCommand = (command: BuiltinInputCommand) => {
    const textarea = inputRef.current;
    if (!textarea) return;

    const cursorPos = textarea.selectionStart;
    const textBeforeCursor = value.slice(0, cursorPos);
    const textAfterCursor = value.slice(cursorPos);
    const lastAtIndex = textBeforeCursor.lastIndexOf("@");

    if (onSelectBuiltinCommand) {
      const newValue = value.slice(0, lastAtIndex) + textAfterCursor;
      onChange(newValue.trimEnd() === "" ? "" : newValue);
      setShowMentions(false);
      onSelectBuiltinCommand(command);

      setTimeout(() => {
        textarea.focus();
        const newCursorPos = Math.max(0, lastAtIndex);
        textarea.setSelectionRange(newCursorPos, newCursorPos);
      }, 0);
      return;
    }

    const newValue =
      value.slice(0, lastAtIndex) + `${command.commandPrefix} ` + textAfterCursor;
    onChange(newValue);
    setShowMentions(false);

    setTimeout(() => {
      textarea.focus();
      const newCursorPos = lastAtIndex + command.commandPrefix.length + 1;
      textarea.setSelectionRange(newCursorPos, newCursorPos);
    }, 0);
  };

  // 处理键盘事件：Escape 关闭，ArrowUp/ArrowDown/Enter 转发给 cmdk
  useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea || !showMentions) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      const composing =
        (e as KeyboardEvent & { isComposing?: boolean }).isComposing ||
        e.key === "Process" ||
        e.keyCode === 229;
      if (composing) {
        return;
      }

      if (e.key === "Escape") {
        setShowMentions(false);
        e.preventDefault();
        return;
      }

      // 转发 ArrowUp/ArrowDown/Enter 给 cmdk Command 根元素
      if (e.key === "ArrowUp" || e.key === "ArrowDown" || e.key === "Enter") {
        e.preventDefault();
        e.stopPropagation();
        const cmdkRoot = commandRef.current;
        if (cmdkRoot) {
          cmdkRoot.dispatchEvent(
            new KeyboardEvent("keydown", {
              key: e.key,
              bubbles: true,
              cancelable: true,
            }),
          );
        }
      }
    };

    textarea.addEventListener("keydown", handleKeyDown);
    return () => textarea.removeEventListener("keydown", handleKeyDown);
  }, [showMentions, inputRef]);

  if (!showMentions) return null;

  return (
    <Popover open={showMentions} onOpenChange={setShowMentions}>
      <PopoverTrigger asChild>
        <div
          data-testid="mention-anchor"
          style={{
            position: "fixed",
            top: panelAnchor.top,
            left: panelAnchor.left,
            width: 1,
            height: 1,
            pointerEvents: "none",
          }}
        />
      </PopoverTrigger>
      <PopoverContent
        ref={popoverRef}
        data-testid="mention-popover-content"
        className="p-0 bg-background border shadow-md"
        align="center"
        side="top"
        sideOffset={12}
        avoidCollisions={false}
        style={{
          width: `${panelAnchor.width}px`,
          maxWidth: "calc(100vw - 24px)",
        }}
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        {showMentions ? (
          <Suspense
            fallback={
              <div className="px-3 py-6 text-center text-sm text-muted-foreground">
                加载中...
              </div>
            }
          >
            <CharacterMentionPanel
              mentionQuery={mentionQuery}
              builtinCommands={filteredBuiltinCommands}
              filteredCharacters={filteredCharacters}
              installedSkills={installedSkills}
              availableSkills={availableSkills}
              commandRef={commandRef}
              onQueryChange={setMentionQuery}
              onSelectBuiltinCommand={handleSelectBuiltinCommand}
              onSelectCharacter={handleSelectCharacter}
              onSelectInstalledSkill={handleSelectInstalledSkill}
              onSelectAvailableSkill={handleSelectAvailableSkill}
              onNavigateToSettings={
                onNavigateToSettings
                  ? () => {
                      setShowMentions(false);
                      onNavigateToSettings();
                    }
                  : undefined
              }
            />
          </Suspense>
        ) : null}
      </PopoverContent>
    </Popover>
  );
}
