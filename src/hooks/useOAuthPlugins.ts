/**
 * @file useOAuthPlugins Hook
 * @description 管理 OAuth Provider 插件的 React Hook
 * @module hooks/useOAuthPlugins
 */

import { useState, useEffect, useCallback } from "react";
import {
  listOAuthPlugins,
  getOAuthPlugin,
  enableOAuthPlugin,
  disableOAuthPlugin,
  installOAuthPlugin,
  uninstallOAuthPlugin,
  checkOAuthPluginUpdates,
  updateOAuthPlugin,
  reloadOAuthPlugins,
  type OAuthPluginInfo,
  type PluginSource,
  type PluginUpdate,
} from "@/lib/api/oauthPlugin";

interface UseOAuthPluginsResult {
  /** 插件列表 */
  plugins: OAuthPluginInfo[];
  /** 加载中状态 */
  loading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 可用更新 */
  updates: PluginUpdate[];
  /** 刷新插件列表 */
  refresh: () => Promise<void>;
  /** 启用插件 */
  enable: (pluginId: string) => Promise<void>;
  /** 禁用插件 */
  disable: (pluginId: string) => Promise<void>;
  /** 安装插件 */
  install: (source: PluginSource) => Promise<boolean>;
  /** 卸载插件 */
  uninstall: (pluginId: string) => Promise<void>;
  /** 更新插件 */
  update: (pluginId: string) => Promise<void>;
  /** 检查更新 */
  checkUpdates: () => Promise<void>;
  /** 重新加载所有插件 */
  reload: () => Promise<void>;
}

/**
 * useOAuthPlugins Hook
 *
 * 管理 OAuth Provider 插件的完整生命周期
 *
 * @example
 * ```tsx
 * function OAuthPluginsPage() {
 *   const { plugins, loading, error, refresh, enable, disable } = useOAuthPlugins();
 *
 *   if (loading) return <Spinner />;
 *   if (error) return <Alert type="error">{error}</Alert>;
 *
 *   return (
 *     <div>
 *       {plugins.map(plugin => (
 *         <PluginCard
 *           key={plugin.id}
 *           plugin={plugin}
 *           onToggle={() => plugin.enabled ? disable(plugin.id) : enable(plugin.id)}
 *         />
 *       ))}
 *     </div>
 *   );
 * }
 * ```
 */
export function useOAuthPlugins(): UseOAuthPluginsResult {
  const [plugins, setPlugins] = useState<OAuthPluginInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [updates, setUpdates] = useState<PluginUpdate[]>([]);

  // 加载插件列表
  const loadPlugins = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const list = await listOAuthPlugins();
      setPlugins(list);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      console.error("[useOAuthPlugins] Failed to load plugins:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  // 首次加载
  useEffect(() => {
    loadPlugins();
  }, [loadPlugins]);

  // 启用插件
  const enable = useCallback(async (pluginId: string) => {
    try {
      await enableOAuthPlugin(pluginId);
      setPlugins((prev) =>
        prev.map((p) => (p.id === pluginId ? { ...p, enabled: true } : p)),
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    }
  }, []);

  // 禁用插件
  const disable = useCallback(async (pluginId: string) => {
    try {
      await disableOAuthPlugin(pluginId);
      setPlugins((prev) =>
        prev.map((p) => (p.id === pluginId ? { ...p, enabled: false } : p)),
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    }
  }, []);

  // 安装插件
  const install = useCallback(
    async (source: PluginSource): Promise<boolean> => {
      try {
        const result = await installOAuthPlugin(source);
        if (result.success) {
          await loadPlugins();
          return true;
        }
        setError(result.error || "安装失败");
        return false;
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        return false;
      }
    },
    [loadPlugins],
  );

  // 卸载插件
  const uninstall = useCallback(async (pluginId: string) => {
    try {
      await uninstallOAuthPlugin(pluginId);
      setPlugins((prev) => prev.filter((p) => p.id !== pluginId));
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    }
  }, []);

  // 更新插件
  const update = useCallback(
    async (pluginId: string) => {
      try {
        await updateOAuthPlugin(pluginId);
        await loadPlugins();
        // 清除该插件的更新记录
        setUpdates((prev) => prev.filter((u) => u.pluginId !== pluginId));
      } catch (e) {
        const msg = e instanceof Error ? e.message : String(e);
        setError(msg);
        throw e;
      }
    },
    [loadPlugins],
  );

  // 检查更新
  const checkUpdates = useCallback(async () => {
    try {
      const result = await checkOAuthPluginUpdates();
      setUpdates(result);
    } catch (e) {
      console.error("[useOAuthPlugins] Failed to check updates:", e);
    }
  }, []);

  // 重新加载所有插件
  const reload = useCallback(async () => {
    try {
      await reloadOAuthPlugins();
      await loadPlugins();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    }
  }, [loadPlugins]);

  return {
    plugins,
    loading,
    error,
    updates,
    refresh: loadPlugins,
    enable,
    disable,
    install,
    uninstall,
    update,
    checkUpdates,
    reload,
  };
}

/**
 * useSingleOAuthPlugin Hook
 *
 * 管理单个 OAuth Provider 插件
 */
export function useSingleOAuthPlugin(pluginId: string): {
  plugin: OAuthPluginInfo | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  toggle: () => Promise<void>;
} {
  const [plugin, setPlugin] = useState<OAuthPluginInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const info = await getOAuthPlugin(pluginId);
      setPlugin(info);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
    } finally {
      setLoading(false);
    }
  }, [pluginId]);

  useEffect(() => {
    load();
  }, [load]);

  const toggle = useCallback(async () => {
    if (!plugin) return;
    try {
      if (plugin.enabled) {
        await disableOAuthPlugin(pluginId);
      } else {
        await enableOAuthPlugin(pluginId);
      }
      setPlugin({ ...plugin, enabled: !plugin.enabled });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setError(msg);
      throw e;
    }
  }, [plugin, pluginId]);

  return {
    plugin,
    loading,
    error,
    refresh: load,
    toggle,
  };
}
