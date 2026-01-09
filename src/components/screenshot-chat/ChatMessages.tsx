/**
 * @file ChatMessages.tsx
 * @description 消息列表组件，显示用户和 AI 的对话消息
 * @module components/screenshot-chat/ChatMessages
 */

import React, { useRef, useEffect } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { ChatMessagesProps, ChatMessage } from "./types";
import "./screenshot-chat.css";

/**
 * 单条消息组件
 */
const MessageItem: React.FC<{ message: ChatMessage }> = ({ message }) => {
  const isUser = message.role === "user";

  return (
    <div
      className={`screenshot-chat-message ${isUser ? "screenshot-chat-message-user" : "screenshot-chat-message-assistant"}`}
    >
      {/* 用户消息显示图片 */}
      {isUser && message.image && (
        <div className="screenshot-chat-message-image">
          <img
            src={`data:${message.image.mediaType};base64,${message.image.data}`}
            alt="截图"
            className="screenshot-chat-message-thumbnail"
          />
        </div>
      )}

      {/* 消息内容 */}
      <div className="screenshot-chat-message-content">
        {message.isThinking ? (
          <div className="screenshot-chat-thinking">
            <span className="screenshot-chat-loading-spinner" />
            <span>{message.thinkingContent || "思考中..."}</span>
          </div>
        ) : isUser ? (
          <p>{message.content}</p>
        ) : (
          <div className="screenshot-chat-markdown">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>
              {message.content}
            </ReactMarkdown>
          </div>
        )}
      </div>

      {/* 时间戳 */}
      <div className="screenshot-chat-message-time">
        {new Date(message.timestamp).toLocaleTimeString("zh-CN", {
          hour: "2-digit",
          minute: "2-digit",
        })}
      </div>
    </div>
  );
};

/**
 * 消息列表组件
 *
 * 显示用户消息和 AI 回复，支持 Markdown 渲染和自动滚动
 *
 * 需求:
 * - 4.5: 悬浮窗口应在可滚动区域显示 AI 回复
 * - 5.4: 当 AI 回复时，悬浮窗口应以 Markdown 格式渲染回复内容
 */
export const ChatMessages: React.FC<ChatMessagesProps> = ({
  messages,
  className = "",
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  // 自动滚动到底部
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [messages]);

  if (messages.length === 0) {
    return (
      <div className={`screenshot-chat-messages ${className}`}>
        <div className="screenshot-chat-placeholder">
          输入问题，开始与 AI 讨论截图内容
        </div>
      </div>
    );
  }

  return (
    <div ref={containerRef} className={`screenshot-chat-messages ${className}`}>
      {messages.map((message) => (
        <MessageItem key={message.id} message={message} />
      ))}
    </div>
  );
};

export default ChatMessages;
