/**
 * 应用主入口组件
 *
 * 管理页面路由和全局状态
 * 支持静态页面和动态插件页面路由
 *
 * _需求: 2.2, 3.2_
 */

import { useState, useEffect } from "react";
import { Sidebar } from "./components/Sidebar";
import { SettingsPage } from "./components/settings";
import { ApiServerPage } from "./components/api-server/ApiServerPage";
import { ProviderPoolPage } from "./components/provider-pool";
import { ConfigManagementPage } from "./components/config/ConfigManagementPage";
import { FlowMonitorPage } from "./pages";
import { ToolsPage } from "./components/tools/ToolsPage";
import { BrowserInterceptorTool } from "./components/tools/browser-interceptor/BrowserInterceptorTool";
import { AgentChatPage } from "./components/agent";
import { PluginUIRenderer } from "./components/plugins/PluginUIRenderer";
import { Toaster } from "./components/ui/sonner";
import { flowEventManager } from "./lib/flowEventManager";

/**
 * 页面类型定义
 *
 * 支持静态页面和动态插件页面
 * - 静态页面: 预定义的页面标识符
 * - 动态插件页面: `plugin:${string}` 格式，如 "plugin:machine-id-tool"
 *
 * _需求: 2.2, 3.2_
 */
type Page =
  | "provider-pool"
  | "config-management"
  | "api-server"
  | "flow-monitor"
  | "agent"
  | "tools"
  | "browser-interceptor"
  | "settings"
  | `plugin:${string}`;

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("api-server");

  // 在应用启动时初始化 Flow 事件订阅
  useEffect(() => {
    flowEventManager.subscribe();
    // 应用卸载时不取消订阅，因为这是全局订阅
  }, []);

  // 页面切换时重置滚动位置
  useEffect(() => {
    const mainElement = document.querySelector("main");
    if (mainElement) {
      mainElement.scrollTop = 0;
    }
  }, [currentPage]);

  /**
   * 渲染当前页面
   *
   * 根据 currentPage 状态渲染对应的页面组件
   * - 静态页面: 直接渲染对应组件
   * - 动态插件页面: 使用 PluginUIRenderer 渲染
   *
   * _需求: 2.2, 3.2_
   */
  const renderPage = () => {
    // 检查是否为动态插件页面 (plugin:xxx 格式)
    if (currentPage.startsWith("plugin:")) {
      const pluginId = currentPage.slice(7); // 移除 "plugin:" 前缀
      return (
        <PluginUIRenderer pluginId={pluginId} onNavigate={setCurrentPage} />
      );
    }

    // 静态页面路由
    switch (currentPage) {
      case "provider-pool":
        return <ProviderPoolPage />;
      case "config-management":
        return <ConfigManagementPage />;
      case "api-server":
        return <ApiServerPage />;
      case "flow-monitor":
        return <FlowMonitorPage />;
      case "agent":
        return <AgentChatPage />;
      case "tools":
        return <ToolsPage onNavigate={setCurrentPage} />;
      case "browser-interceptor":
        return <BrowserInterceptorTool onNavigate={setCurrentPage} />;
      case "settings":
        return <SettingsPage />;
      default:
        return <ApiServerPage />;
    }
  };

  return (
    <div className="flex h-screen bg-background">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-auto p-6">{renderPage()}</main>
      <Toaster />
    </div>
  );
}

export default App;
