/**
 * @file RootRouter.tsx
 * @description 根路由组件 - 根据 URL 路径渲染对应的组件
 */

import App from "./App";
import { ScreenshotChatPage } from "./pages/screenshot-chat";
import { Toaster } from "./components/ui/sonner";

/**
 * 根据 URL 路径渲染对应的组件
 *
 * - /screenshot-chat: 截图对话悬浮窗口（独立 Tauri 窗口）
 * - 其他: 主应用
 */
export function RootRouter() {
  const pathname = window.location.pathname;

  // 截图对话悬浮窗口路由
  if (pathname === "/screenshot-chat") {
    return <ScreenshotChatPage />;
  }

  // 默认渲染主应用
  return (
    <>
      <App />
      <Toaster />
    </>
  );
}
