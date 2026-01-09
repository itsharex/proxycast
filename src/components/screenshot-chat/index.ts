/**
 * @file index.ts
 * @description 截图对话模块导出入口
 * @module components/screenshot-chat
 */

// 类型导出
export type {
  ScreenshotChatConfig,
  MessageImage,
  ChatMessage,
  ScreenshotChatState,
  UseScreenshotChatReturn,
  ScreenshotPreviewProps,
  ChatInputProps,
  ChatMessagesProps,
  ScreenshotChatWindowProps,
} from "./types";

export type { ShortcutSettingsProps } from "./ShortcutSettings";

// 组件导出
export { ScreenshotPreview } from "./ScreenshotPreview";
export { ChatInput } from "./ChatInput";
export { ChatMessages } from "./ChatMessages";
export { ScreenshotChatWindow } from "./ScreenshotChatWindow";
export { ShortcutSettings } from "./ShortcutSettings";

// Hook 导出
export { useScreenshotChat, readImageAsBase64 } from "./useScreenshotChat";

// 默认导出主组件
export { ScreenshotChatWindow as default } from "./ScreenshotChatWindow";
