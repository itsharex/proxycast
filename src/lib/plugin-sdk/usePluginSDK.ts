/**
 * @file usePluginSDK Hook
 * @description 在 React 组件中使用 Plugin SDK 的 Hook
 * @module lib/plugin-sdk/usePluginSDK
 */

import { useMemo, useCallback, useEffect, useState } from "react";
import { getPluginSDK } from "./sdk";
import type { ProxyCastPluginSDK, PluginId, CredentialInfo } from "./types";

/**
 * usePluginSDK Hook
 *
 * 在 React 组件中获取 Plugin SDK 实例
 *
 * @param pluginId 插件 ID
 * @returns SDK 实例和相关状态
 *
 * @example
 * ```tsx
 * function MyPluginUI({ pluginId }: { pluginId: string }) {
 *   const { sdk, credentials, loading, error, refresh } = usePluginSDK(pluginId);
 *
 *   if (loading) return <Spinner />;
 *   if (error) return <Alert type="error">{error}</Alert>;
 *
 *   return (
 *     <div>
 *       {credentials.map(cred => (
 *         <CredentialCard key={cred.id} credential={cred} />
 *       ))}
 *       <button onClick={refresh}>Refresh</button>
 *     </div>
 *   );
 * }
 * ```
 */
export function usePluginSDK(pluginId: PluginId): {
  /** SDK 实例 */
  sdk: ProxyCastPluginSDK;
  /** 凭证列表 */
  credentials: CredentialInfo[];
  /** 加载中状态 */
  loading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 刷新凭证列表 */
  refresh: () => Promise<void>;
} {
  const sdk = useMemo(() => getPluginSDK(pluginId), [pluginId]);

  const [credentials, setCredentials] = useState<CredentialInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载凭证列表
  const loadCredentials = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const list = await sdk.credential.list();
      setCredentials(list);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
      console.error(`[usePluginSDK] Failed to load credentials:`, e);
    } finally {
      setLoading(false);
    }
  }, [sdk]);

  // 首次加载
  useEffect(() => {
    loadCredentials();
  }, [loadCredentials]);

  // 订阅凭证变更事件
  useEffect(() => {
    const unsubscribe = sdk.events.on("credential:changed", () => {
      loadCredentials();
    });

    return () => {
      unsubscribe();
    };
  }, [sdk, loadCredentials]);

  return {
    sdk,
    credentials,
    loading,
    error,
    refresh: loadCredentials,
  };
}

/**
 * usePluginConfig Hook
 *
 * 管理插件配置的 Hook
 *
 * @param pluginId 插件 ID
 * @returns 配置对象和更新方法
 */
export function usePluginConfig<T = Record<string, unknown>>(
  pluginId: PluginId,
): {
  config: T | null;
  loading: boolean;
  error: string | null;
  updateConfig: (newConfig: Partial<T>) => Promise<void>;
  refreshConfig: () => Promise<void>;
} {
  const sdk = useMemo(() => getPluginSDK(pluginId), [pluginId]);

  const [config, setConfig] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadConfig = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const cfg = await sdk.config.get<T>();
      setConfig(cfg);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [sdk]);

  const updateConfig = useCallback(
    async (newConfig: Partial<T>) => {
      try {
        const merged = { ...config, ...newConfig } as Record<string, unknown>;
        await sdk.config.set(merged);
        setConfig(merged as T);
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        setError(errorMessage);
        throw e;
      }
    },
    [sdk, config],
  );

  useEffect(() => {
    loadConfig();
  }, [loadConfig]);

  return {
    config,
    loading,
    error,
    updateConfig,
    refreshConfig: loadConfig,
  };
}

/**
 * usePluginEvents Hook
 *
 * 订阅插件事件的 Hook
 *
 * @param pluginId 插件 ID
 * @param event 事件名称
 * @param callback 回调函数
 */
export function usePluginEvents<T = unknown>(
  pluginId: PluginId,
  event: string,
  callback: (data: T) => void,
): void {
  const sdk = useMemo(() => getPluginSDK(pluginId), [pluginId]);

  useEffect(() => {
    const unsubscribe = sdk.events.on<T>(event, callback);
    return () => {
      unsubscribe();
    };
  }, [sdk, event, callback]);
}

/**
 * usePluginStorage Hook
 *
 * 管理插件存储的 Hook
 *
 * @param pluginId 插件 ID
 * @param key 存储键
 * @param defaultValue 默认值
 * @returns 存储值和更新方法
 */
export function usePluginStorage(
  pluginId: PluginId,
  key: string,
  defaultValue?: string,
): {
  value: string | null;
  loading: boolean;
  error: string | null;
  setValue: (value: string) => Promise<void>;
  deleteValue: () => Promise<void>;
} {
  const sdk = useMemo(() => getPluginSDK(pluginId), [pluginId]);

  const [value, setValueState] = useState<string | null>(defaultValue ?? null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const load = async () => {
      try {
        setLoading(true);
        setError(null);
        const stored = await sdk.storage.get(key);
        setValueState(stored ?? defaultValue ?? null);
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        setError(errorMessage);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, [sdk, key, defaultValue]);

  const setValue = useCallback(
    async (newValue: string) => {
      try {
        await sdk.storage.set(key, newValue);
        setValueState(newValue);
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        setError(errorMessage);
        throw e;
      }
    },
    [sdk, key],
  );

  const deleteValue = useCallback(async () => {
    try {
      await sdk.storage.delete(key);
      setValueState(null);
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setError(errorMessage);
      throw e;
    }
  }, [sdk, key]);

  return {
    value,
    loading,
    error,
    setValue,
    deleteValue,
  };
}
