/**
 * @file RootRouter.tsx
 * @description 根路由组件 - 根据 URL 路径渲染对应的组件
 */

import App from "./App";
import { SmartInputPage } from "./pages/smart-input";
import { UpdateNotificationPage } from "./pages/update-notification";
import { Toaster } from "./components/ui/sonner";
import { AppCrashBoundary } from "./components/layout/AppCrashBoundary";

/**
 * 根据 URL 路径渲染对应的组件
 *
 * - /smart-input: 截图对话悬浮窗口（独立 Tauri 窗口，支持语音模式）
 * - /update-notification: 更新提醒悬浮窗口（独立 Tauri 窗口）
 * - 其他: 主应用
 */
export function RootRouter() {
  const pathname = window.location.pathname;

  // 截图对话悬浮窗口路由（也用于语音输入）
  if (pathname === "/smart-input") {
    return (
      <AppCrashBoundary>
        <SmartInputPage />
      </AppCrashBoundary>
    );
  }

  // 更新提醒悬浮窗口路由
  if (pathname === "/update-notification") {
    return (
      <AppCrashBoundary>
        <UpdateNotificationPage />
      </AppCrashBoundary>
    );
  }

  // 默认渲染主应用
  return (
    <AppCrashBoundary>
      <App />
      <Toaster />
    </AppCrashBoundary>
  );
}
