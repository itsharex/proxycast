/**
 * @file 插件 UI 容器组件
 * @description 封装插件 UI 渲染的完整容器，包含加载状态和错误处理
 * @module lib/plugin-ui/PluginUIContainer
 */

import React from "react";
import { Loader2, AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { PluginUIRenderer } from "./PluginUIRenderer";
import { usePluginUI } from "./usePluginUI";
import type { PluginId } from "./types";

interface PluginUIContainerProps {
  /** 插件 ID */
  pluginId: PluginId;
  /** 自定义类名 */
  className?: string;
  /** 空状态提示 */
  emptyMessage?: string;
}

/**
 * 插件 UI 容器
 * 自动管理插件 UI 的加载、渲染和错误处理
 */
export const PluginUIContainer: React.FC<PluginUIContainerProps> = ({
  pluginId,
  className,
  emptyMessage = "该插件没有提供 UI",
}) => {
  const { surfaces, loading, error, handleAction, refresh } = usePluginUI({
    pluginId,
  });

  // 加载状态
  if (loading) {
    return (
      <div className={`flex items-center justify-center p-8 ${className}`}>
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载插件 UI...</span>
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div
        className={`flex flex-col items-center justify-center p-8 ${className}`}
      >
        <AlertCircle className="h-8 w-8 text-red-500 mb-2" />
        <p className="text-red-600 mb-4">{error}</p>
        <Button variant="outline" size="sm" onClick={refresh}>
          <RefreshCw className="h-4 w-4 mr-2" />
          重试
        </Button>
      </div>
    );
  }

  // 空状态
  if (surfaces.length === 0) {
    return (
      <div
        className={`flex items-center justify-center p-8 text-muted-foreground ${className}`}
      >
        {emptyMessage}
      </div>
    );
  }

  // 渲染所有 Surface
  return (
    <div className={`space-y-4 ${className}`}>
      {surfaces.map((surface) => (
        <PluginUIRenderer
          key={surface.surfaceId}
          surface={surface}
          onAction={handleAction}
        />
      ))}
    </div>
  );
};

export default PluginUIContainer;
