/**
 * 插件 UI 渲染器组件
 *
 * 根据 pluginId 渲染对应的插件 UI 组件
 * 支持内置插件组件映射和错误处理
 *
 * _需求: 3.2_
 */

import React from "react";
import { AlertCircle, Package } from "lucide-react";
import { MachineIdTool } from "@/components/tools/machine-id/MachineIdTool";

/**
 * 页面类型定义
 * 支持静态页面和动态插件页面
 */
export type Page =
  | "provider-pool"
  | "config-management"
  | "api-server"
  | "flow-monitor"
  | "agent"
  | "tools"
  | "browser-interceptor"
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
 *
 * 当插件 UI 组件加载失败时显示友好的错误提示
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
 *
 * 当请求的插件不存在时显示提示
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
 * 内置插件组件映射
 *
 * 将插件 ID 映射到对应的 React 组件
 * 目前支持 machine-id-tool 插件
 */
const builtinPluginComponents: Record<
  string,
  React.ComponentType<{ onNavigate: (page: Page) => void }>
> = {
  "machine-id-tool": MachineIdTool,
};

/**
 * 插件 UI 渲染器
 *
 * 根据 pluginId 渲染对应的插件 UI 组件
 * - 对于内置插件，直接渲染对应的 React 组件
 * - 对于未知插件，显示错误提示
 *
 * @param pluginId - 插件 ID
 * @param onNavigate - 页面导航回调
 */
export function PluginUIRenderer({
  pluginId,
  onNavigate,
}: PluginUIRendererProps) {
  // 查找内置插件组件
  const Component = builtinPluginComponents[pluginId];

  if (Component) {
    try {
      return <Component onNavigate={onNavigate} />;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : "未知错误";
      return <PluginUIError pluginId={pluginId} error={errorMessage} />;
    }
  }

  // 插件未找到
  return <PluginNotFound pluginId={pluginId} />;
}

export default PluginUIRenderer;
