/**
 * 插件 UI 渲染器组件
 *
 * 根据 pluginId 渲染对应的插件 UI 组件
 * 支持内置插件组件映射、动态加载外部插件和错误处理
 *
 * _需求: 3.2_
 */

import React, { useState, useEffect } from "react";
import { AlertCircle, Package, Loader2 } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { MachineIdTool } from "@/components/tools/machine-id/MachineIdTool";
import { BrowserInterceptorTool } from "@/components/tools/browser-interceptor/BrowserInterceptorTool";
import { FlowMonitorPage } from "@/pages";
import { ConfigManagementPage } from "@/components/config/ConfigManagementPage";
import { TerminalPage } from "@/components/terminal";
import { PluginUIRenderer as DynamicPluginRenderer } from "@/lib/plugin-loader/PluginUIRenderer";
import { usePluginSDK } from "@/lib/plugin-sdk";

/**
 * 页面类型定义
 * 支持静态页面和动态插件页面
 */
export type Page =
  | "provider-pool"
  | "api-server"
  | "agent"
  | "tools"
  | "plugins"
  | "settings"
  | `plugin:${string}`;

/**
 * PluginUIRenderer 组件属性
 */
interface PluginUIRendererProps {
  /** 插件 ID */
  pluginId: string;
  /** 页面导航回调 */
  onNavigate: (page: Page) => void;
}

/**
 * 插件 UI 加载错误组件
 */
function PluginUIError({
  pluginId,
  error,
}: {
  pluginId: string;
  error: string;
}) {
  return (
    <div className="flex flex-col items-center justify-center h-96 space-y-4">
      <div className="p-4 bg-red-50 dark:bg-red-900/20 rounded-full">
        <AlertCircle className="w-12 h-12 text-red-500" />
      </div>
      <div className="text-center space-y-2">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
          插件 UI 加载失败
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          无法加载插件 "{pluginId}" 的用户界面
        </p>
        <p className="text-sm text-gray-500 dark:text-gray-500">{error}</p>
      </div>
    </div>
  );
}

/**
 * 插件未找到组件
 */
function PluginNotFound({ pluginId }: { pluginId: string }) {
  return (
    <div className="flex flex-col items-center justify-center h-96 space-y-4">
      <div className="p-4 bg-gray-100 dark:bg-gray-800 rounded-full">
        <Package className="w-12 h-12 text-gray-400" />
      </div>
      <div className="text-center space-y-2">
        <h2 className="text-xl font-semibold text-gray-900 dark:text-gray-100">
          插件未找到
        </h2>
        <p className="text-gray-600 dark:text-gray-400">
          插件 "{pluginId}" 未安装或不存在
        </p>
        <p className="text-sm text-gray-500 dark:text-gray-500">
          请检查插件是否已正确安装
        </p>
      </div>
    </div>
  );
}

/**
 * 加载中组件
 */
function PluginLoading() {
  return (
    <div className="flex flex-col items-center justify-center h-96 space-y-4">
      <Loader2 className="w-12 h-12 text-primary animate-spin" />
      <p className="text-gray-600 dark:text-gray-400">加载插件中...</p>
    </div>
  );
}

/**
 * 内置插件组件映射
 */
const builtinPluginComponents: Record<
  string,
  React.ComponentType<{ onNavigate?: (page: Page) => void }>
> = {
  "machine-id-tool": MachineIdTool,
  "browser-interception": BrowserInterceptorTool,
  "flow-monitor": FlowMonitorPage,
  "config-switch": ConfigManagementPage,
  "terminal-plugin": TerminalPage,
};

/**
 * 已安装插件信息
 */
interface InstalledPlugin {
  id: string;
  name: string;
  install_path: string;
  has_ui: boolean;
  ui_entry?: string;
}

/**
 * 动态插件渲染器
 * 用于加载外部安装的插件 UI
 */
function DynamicPluginUIRenderer({
  pluginId,
  pluginsDir,
  uiEntry,
}: {
  pluginId: string;
  pluginsDir: string;
  uiEntry?: string;
}) {
  const { sdk } = usePluginSDK(pluginId);

  return (
    <DynamicPluginRenderer
      pluginsDir={pluginsDir}
      pluginId={pluginId}
      uiEntry={uiEntry || "dist/index.js"}
      sdk={sdk}
      className="h-full w-full"
      fallback={<PluginNotFound pluginId={pluginId} />}
    />
  );
}

/**
 * 插件 UI 渲染器
 *
 * 根据 pluginId 渲染对应的插件 UI 组件
 * - 对于内置插件，直接渲染对应的 React 组件
 * - 对于外部安装的插件，动态加载其 UI
 * - 对于未知插件，显示错误提示
 */
export function PluginUIRenderer({
  pluginId,
  onNavigate,
}: PluginUIRendererProps) {
  const [loading, setLoading] = useState(true);
  const [pluginInfo, setPluginInfo] = useState<InstalledPlugin | null>(null);
  const [pluginsDir, setPluginsDir] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  // 查找内置插件组件
  const BuiltinComponent = builtinPluginComponents[pluginId];

  // 对于非内置插件，检查是否已安装并有 UI
  useEffect(() => {
    // 如果是内置插件，跳过检查
    if (BuiltinComponent) {
      setLoading(false);
      return;
    }

    async function checkPlugin() {
      setLoading(true);
      setError(null);

      try {
        // 获取插件目录
        const dir = await invoke<string>("get_plugins_dir");
        setPluginsDir(dir);

        // 检查插件是否已安装
        const installed = await invoke<boolean>("is_plugin_installed", {
          pluginId,
        });

        if (!installed) {
          setPluginInfo(null);
          setLoading(false);
          return;
        }

        // 获取插件信息
        const plugins = await invoke<InstalledPlugin[]>(
          "list_installed_plugins",
        );
        const plugin = plugins.find((p) => p.id === pluginId);

        if (plugin) {
          setPluginInfo(plugin);
        } else {
          setPluginInfo(null);
        }
      } catch (err) {
        console.error("检查插件失败:", err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setLoading(false);
      }
    }

    checkPlugin();
  }, [pluginId, BuiltinComponent]);

  // 加载中
  if (loading) {
    return <PluginLoading />;
  }

  // 如果是内置插件，直接渲染
  if (BuiltinComponent) {
    try {
      return <BuiltinComponent onNavigate={onNavigate} />;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "未知错误";
      return <PluginUIError pluginId={pluginId} error={errorMessage} />;
    }
  }

  // 错误
  if (error) {
    return <PluginUIError pluginId={pluginId} error={error} />;
  }

  // 插件未安装
  if (!pluginInfo) {
    return <PluginNotFound pluginId={pluginId} />;
  }

  // 动态加载插件 UI
  return (
    <DynamicPluginUIRenderer
      pluginId={pluginId}
      pluginsDir={pluginsDir}
      uiEntry={pluginInfo.ui_entry}
    />
  );
}

export default PluginUIRenderer;
