/**
 * 角色引用组件
 *
 * 在输入框中检测 @ 符号，显示角色列表供选择
 */

import React, { useState, useEffect, useMemo, useRef } from "react";
import { User } from "lucide-react";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { Character } from "@/lib/api/memory";

interface CharacterMentionProps {
  /** 角色列表 */
  characters: Character[];
  /** 输入框 ref */
  inputRef: React.RefObject<HTMLTextAreaElement>;
  /** 当前输入值 */
  value: string;
  /** 输入值变更回调 */
  onChange: (value: string) => void;
  /** 选择角色回调 */
  onSelectCharacter?: (character: Character) => void;
}

export function CharacterMention({
  characters,
  inputRef,
  value,
  onChange,
  onSelectCharacter,
}: CharacterMentionProps) {
  const [showMentions, setShowMentions] = useState(false);
  const [mentionQuery, setMentionQuery] = useState("");
  const [cursorPosition, setCursorPosition] = useState({ top: 0, left: 0 });
  const popoverRef = useRef<HTMLDivElement>(null);

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

  // 检测 @ 符号
  useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea) return;

    const handleInput = () => {
      const cursorPos = textarea.selectionStart;
      const textBeforeCursor = value.slice(0, cursorPos);
      const lastAtIndex = textBeforeCursor.lastIndexOf("@");

      // 检查是否在 @ 后面输入
      if (lastAtIndex !== -1) {
        const textAfterAt = textBeforeCursor.slice(lastAtIndex + 1);
        // 如果 @ 后面没有空格，说明正在输入角色名
        if (!textAfterAt.includes(" ") && !textAfterAt.includes("\n")) {
          setMentionQuery(textAfterAt);
          setShowMentions(true);

          // 计算弹窗位置
          const rect = textarea.getBoundingClientRect();
          const lineHeight = parseInt(
            window.getComputedStyle(textarea).lineHeight,
          );
          const lines = textBeforeCursor.split("\n");
          const currentLine = lines.length - 1;
          const top = rect.top + currentLine * lineHeight - textarea.scrollTop;
          const left = rect.left + 10; // 简单的左边距

          setCursorPosition({ top, left });
          return;
        }
      }

      setShowMentions(false);
    };

    textarea.addEventListener("input", handleInput);
    textarea.addEventListener("click", handleInput);
    textarea.addEventListener("keyup", handleInput);

    return () => {
      textarea.removeEventListener("input", handleInput);
      textarea.removeEventListener("click", handleInput);
      textarea.removeEventListener("keyup", handleInput);
    };
  }, [value, inputRef]);

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

  // 处理键盘事件
  useEffect(() => {
    const textarea = inputRef.current;
    if (!textarea || !showMentions) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setShowMentions(false);
        e.preventDefault();
      }
    };

    textarea.addEventListener("keydown", handleKeyDown);
    return () => textarea.removeEventListener("keydown", handleKeyDown);
  }, [showMentions, inputRef]);

  if (!showMentions || characters.length === 0) return null;

  return (
    <Popover open={showMentions} onOpenChange={setShowMentions}>
      <PopoverTrigger asChild>
        <div
          style={{
            position: "fixed",
            top: cursorPosition.top - 200, // 弹窗在输入框上方
            left: cursorPosition.left,
            width: 1,
            height: 1,
            pointerEvents: "none",
          }}
        />
      </PopoverTrigger>
      <PopoverContent
        ref={popoverRef}
        className="w-80 p-0"
        align="start"
        side="top"
        onOpenAutoFocus={(e) => e.preventDefault()}
      >
        <Command>
          <CommandInput
            placeholder="搜索角色..."
            value={mentionQuery}
            onValueChange={setMentionQuery}
          />
          <CommandList>
            <CommandEmpty>没有找到角色</CommandEmpty>
            <CommandGroup heading="选择角色">
              {filteredCharacters.map((character) => (
                <CommandItem
                  key={character.id}
                  onSelect={() => handleSelectCharacter(character)}
                  className="cursor-pointer"
                >
                  <User className="mr-2 h-4 w-4" />
                  <div className="flex-1">
                    <div className="font-medium">{character.name}</div>
                    {character.description && (
                      <div className="text-xs text-muted-foreground line-clamp-1">
                        {character.description}
                      </div>
                    )}
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
