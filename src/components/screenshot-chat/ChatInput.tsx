/**
 * @file ChatInput.tsx
 * @description 聊天输入框组件，支持文本输入和发送
 * @module components/screenshot-chat/ChatInput
 */

import React, { useCallback, useRef, useEffect } from "react";
import type { ChatInputProps } from "./types";

/**
 * 聊天输入框组件
 *
 * 提供文本输入框和发送按钮，支持 Enter 键发送
 *
 * 需求:
 * - 4.3: 悬浮窗口应提供文本输入框供用户输入问题
 * - 4.4: 当用户按下 Enter 或点击发送时，悬浮窗口应将图片和文本发送给 AI
 */
export const ChatInput: React.FC<ChatInputProps> = ({
  value,
  onChange,
  onSend,
  disabled = false,
  isLoading = false,
  placeholder = "输入问题...",
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  // 自动聚焦输入框
  useEffect(() => {
    if (!disabled && inputRef.current) {
      inputRef.current.focus();
    }
  }, [disabled]);

  // 处理键盘事件
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Enter" && !e.shiftKey && !disabled && !isLoading) {
        e.preventDefault();
        if (value.trim()) {
          onSend();
        }
      }
    },
    [value, onSend, disabled, isLoading],
  );

  // 处理输入变化
  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      onChange(e.target.value);
    },
    [onChange],
  );

  // 处理发送按钮点击
  const handleSendClick = useCallback(() => {
    if (value.trim() && !disabled && !isLoading) {
      onSend();
    }
  }, [value, onSend, disabled, isLoading]);

  const canSend = value.trim() && !disabled && !isLoading;

  return (
    <div className="screenshot-chat-input-area">
      <input
        ref={inputRef}
        type="text"
        className="screenshot-chat-input"
        value={value}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled || isLoading}
        autoFocus
      />
      <button
        className="screenshot-chat-send-btn"
        onClick={handleSendClick}
        disabled={!canSend}
        title="发送 (Enter)"
      >
        {isLoading ? (
          <span
            className="screenshot-chat-loading-spinner"
            style={{ width: 16, height: 16 }}
          />
        ) : (
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <line x1="22" y1="2" x2="11" y2="13" />
            <polygon points="22 2 15 22 11 13 2 9 22 2" />
          </svg>
        )}
      </button>
    </div>
  );
};

export default ChatInput;
