/**
 * @file 插件 UI 渲染容器
 * @description 加载并渲染插件的 React 组件
 * @module lib/plugin-loader/PluginUIRenderer
 */

import React, { useState, useEffect, Suspense } from "react";
import { Loader2, AlertCircle, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { loadPluginUI, getPluginUIPath, clearPluginCache } from "./index";
import type { PluginModule } from "./index";
import type { ProxyCastPluginSDK as PluginSDK } from "@/lib/plugin-sdk/types";

interface PluginUIRendererProps {
  /** 插件目录 */
  pluginsDir: string;
  /** 插件 ID */
  pluginId: string;
  /** UI 入口文件（相对路径） */
  uiEntry?: string;
  /** 插件 SDK */
  sdk: PluginSDK;
  /** 自定义类名 */
  className?: string;
  /** 加载失败时的回退组件 */
  fallback?: React.ReactNode;
}

/**
 * 插件 UI 渲染容器
 *
 * 负责：
 * 1. 动态加载插件的 React 组件
 * 2. 注入 SDK 和其他依赖
 * 3. 处理加载状态和错误
 */
export function PluginUIRenderer({
  pluginsDir,
  pluginId,
  uiEntry = "dist/index.js",
  sdk,
  className,
  fallback,
}: PluginUIRendererProps) {
  const [module, setModule] = useState<PluginModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const pluginPath = getPluginUIPath(pluginsDir, pluginId, uiEntry);

  // 加载插件
  useEffect(() => {
    let cancelled = false;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        const loadedModule = await loadPluginUI(pluginPath);
        if (cancelled) return;

        if (loadedModule) {
          setModule(loadedModule);
        } else {
          setError("插件加载失败：没有找到有效的组件导出");
        }
      } catch (err) {
        if (cancelled) return;
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    load();

    return () => {
      cancelled = true;
    };
  }, [pluginPath]);

  // 重新加载
  const handleReload = () => {
    clearPluginCache(pluginPath);
    setModule(null);
    setLoading(true);
    setError(null);

    loadPluginUI(pluginPath)
      .then((loadedModule) => {
        if (loadedModule) {
          setModule(loadedModule);
        } else {
          setError("插件加载失败");
        }
      })
      .catch((err) => {
        setError(err instanceof Error ? err.message : String(err));
      })
      .finally(() => {
        setLoading(false);
      });
  };

  // 加载中
  if (loading) {
    return (
      <div className={`flex items-center justify-center p-8 ${className}`}>
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载插件 UI...</span>
      </div>
    );
  }

  // 错误
  if (error) {
    // 检查是否是文件不存在的错误
    const isFileNotFound =
      error.includes("读取插件 UI 文件失败") ||
      error.includes("No such file") ||
      error.includes("not found") ||
      error.includes("没有找到有效的组件导出") ||
      error.includes("插件加载失败");

    if (isFileNotFound) {
      // UI 文件不存在时显示友好提示
      return fallback ? (
        <>{fallback}</>
      ) : (
        <div
          className={`flex flex-col items-center justify-center p-8 text-muted-foreground ${className}`}
        >
          <AlertCircle className="h-8 w-8 mb-2 opacity-50" />
          <p className="text-center text-sm">该插件暂无 UI 界面</p>
          <p className="text-center text-xs mt-1 opacity-70">
            请通过命令行或 API 使用此插件
          </p>
        </div>
      );
    }

    return (
      <div
        className={`flex flex-col items-center justify-center p-8 ${className}`}
      >
        <AlertCircle className="h-8 w-8 text-red-500 mb-2" />
        <p className="text-red-600 mb-4 text-center">{error}</p>
        <Button variant="outline" size="sm" onClick={handleReload}>
          <RefreshCw className="h-4 w-4 mr-2" />
          重试
        </Button>
      </div>
    );
  }

  // 没有模块
  if (!module) {
    return fallback ? <>{fallback}</> : null;
  }

  // 渲染插件组件
  const PluginComponent = module.default;

  return (
    <div className={`h-full w-full ${className || ""}`}>
      <Suspense
        fallback={
          <div className="flex items-center justify-center p-8 h-full">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        }
      >
        <PluginComponent sdk={sdk} pluginId={pluginId} />
      </Suspense>
    </div>
  );
}

export default PluginUIRenderer;
