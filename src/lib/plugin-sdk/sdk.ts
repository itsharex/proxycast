/**
 * @file ProxyCast Plugin SDK 实现
 * @description 提供给 OAuth Provider 插件 UI 使用的 SDK 实现
 * @module lib/plugin-sdk/sdk
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  ProxyCastPluginSDK,
  PluginId,
  DatabaseApi,
  HttpApi,
  CryptoApi,
  NotificationApi,
  EventsApi,
  StorageApi,
  CredentialApi,
  PluginConfigApi,
  QueryResult,
  ExecuteResult,
  HttpRequestOptions,
  HttpResponse,
  EventCallback,
  Unsubscribe,
  CredentialInfo,
  CredentialId,
} from "./types";

// ============================================================================
// 事件总线
// ============================================================================

type EventListeners = Map<string, Set<EventCallback>>;

class PluginEventBus {
  private listeners: EventListeners = new Map();

  emit(event: string, data?: unknown): void {
    const eventListeners = this.listeners.get(event);
    if (eventListeners) {
      eventListeners.forEach((callback) => {
        try {
          callback(data);
        } catch (error) {
          console.error(
            `[PluginEventBus] Error in event handler for '${event}':`,
            error,
          );
        }
      });
    }
  }

  on<T = unknown>(event: string, callback: EventCallback<T>): Unsubscribe {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    const eventListeners = this.listeners.get(event)!;
    eventListeners.add(callback as EventCallback);

    return () => {
      eventListeners.delete(callback as EventCallback);
      if (eventListeners.size === 0) {
        this.listeners.delete(event);
      }
    };
  }

  once<T = unknown>(event: string, callback: EventCallback<T>): void {
    const unsubscribe = this.on<T>(event, (data) => {
      unsubscribe();
      callback(data);
    });
  }

  clear(): void {
    this.listeners.clear();
  }
}

// 全局事件总线
const globalEventBus = new PluginEventBus();

// ============================================================================
// Database API 实现
// ============================================================================

function createDatabaseApi(pluginId: PluginId): DatabaseApi {
  return {
    async query<T = Record<string, unknown>>(
      sql: string,
      params?: unknown[],
    ): Promise<QueryResult<T>> {
      try {
        const result = await invoke<QueryResult<T>>("plugin_database_query", {
          pluginId,
          sql,
          params: params || [],
        });
        return result;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Database query error:`, error);
        throw error;
      }
    },

    async execute(sql: string, params?: unknown[]): Promise<ExecuteResult> {
      try {
        const result = await invoke<ExecuteResult>("plugin_database_execute", {
          pluginId,
          sql,
          params: params || [],
        });
        return result;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Database execute error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// HTTP API 实现
// ============================================================================

function createHttpApi(pluginId: PluginId): HttpApi {
  return {
    async request(
      url: string,
      options?: HttpRequestOptions,
    ): Promise<HttpResponse> {
      try {
        const result = await invoke<HttpResponse>("plugin_http_request", {
          pluginId,
          url,
          method: options?.method || "GET",
          headers: options?.headers || {},
          body: options?.body,
          timeoutMs: options?.timeoutMs || 30000,
        });
        return result;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] HTTP request error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// Crypto API 实现
// ============================================================================

function createCryptoApi(pluginId: PluginId): CryptoApi {
  return {
    async encrypt(data: string): Promise<string> {
      try {
        const result = await invoke<{ encrypted: string }>(
          "plugin_crypto_encrypt",
          {
            pluginId,
            data,
          },
        );
        return result.encrypted;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Crypto encrypt error:`, error);
        throw error;
      }
    },

    async decrypt(data: string): Promise<string> {
      try {
        const result = await invoke<{ decrypted: string }>(
          "plugin_crypto_decrypt",
          {
            pluginId,
            data,
          },
        );
        return result.decrypted;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Crypto decrypt error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// Notification API 实现
// ============================================================================

function createNotificationApi(pluginId: PluginId): NotificationApi {
  const notify = (level: string, message: string) => {
    // 发送到前端通知系统
    globalEventBus.emit("notification", { level, message, pluginId });

    // 同时调用后端记录日志
    invoke("plugin_notification", { pluginId, level, message }).catch(
      (error) => {
        console.error(`[Plugin ${pluginId}] Notification error:`, error);
      },
    );
  };

  return {
    success(message: string): void {
      notify("success", message);
    },
    error(message: string): void {
      notify("error", message);
    },
    info(message: string): void {
      notify("info", message);
    },
    warning(message: string): void {
      notify("warning", message);
    },
  };
}

// ============================================================================
// Events API 实现
// ============================================================================

function createEventsApi(pluginId: PluginId): EventsApi {
  // 创建插件专属的事件前缀
  const prefix = `plugin:${pluginId}:`;

  return {
    emit(event: string, data?: unknown): void {
      globalEventBus.emit(`${prefix}${event}`, data);

      // 如果是跨插件事件，发送到后端
      if (event.startsWith("global:")) {
        invoke("plugin_event_emit", { pluginId, event, data }).catch(
          (error) => {
            console.error(`[Plugin ${pluginId}] Event emit error:`, error);
          },
        );
      }
    },

    on<T = unknown>(event: string, callback: EventCallback<T>): Unsubscribe {
      return globalEventBus.on<T>(`${prefix}${event}`, callback);
    },

    once<T = unknown>(event: string, callback: EventCallback<T>): void {
      globalEventBus.once<T>(`${prefix}${event}`, callback);
    },
  };
}

// ============================================================================
// Storage API 实现
// ============================================================================

function createStorageApi(pluginId: PluginId): StorageApi {
  return {
    async get(key: string): Promise<string | null> {
      try {
        const result = await invoke<{ value: string | null }>(
          "plugin_storage_get",
          {
            pluginId,
            key,
          },
        );
        return result.value;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Storage get error:`, error);
        throw error;
      }
    },

    async set(key: string, value: string): Promise<void> {
      try {
        await invoke("plugin_storage_set", { pluginId, key, value });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Storage set error:`, error);
        throw error;
      }
    },

    async delete(key: string): Promise<void> {
      try {
        await invoke("plugin_storage_delete", { pluginId, key });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Storage delete error:`, error);
        throw error;
      }
    },

    async keys(): Promise<string[]> {
      try {
        const result = await invoke<{ keys: string[] }>("plugin_storage_keys", {
          pluginId,
        });
        return result.keys;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Storage keys error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// Credential API 实现
// ============================================================================

function createCredentialApi(pluginId: PluginId): CredentialApi {
  return {
    async list(): Promise<CredentialInfo[]> {
      try {
        const result = await invoke<{ credentials: CredentialInfo[] }>(
          "plugin_credential_list",
          { pluginId },
        );
        return result.credentials;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential list error:`, error);
        throw error;
      }
    },

    async get(id: CredentialId): Promise<CredentialInfo | null> {
      try {
        const result = await invoke<{ credential: CredentialInfo | null }>(
          "plugin_credential_get",
          { pluginId, credentialId: id },
        );
        return result.credential;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential get error:`, error);
        throw error;
      }
    },

    async create(
      authType: string,
      config: Record<string, unknown>,
    ): Promise<CredentialId> {
      try {
        const result = await invoke<{ credentialId: CredentialId }>(
          "plugin_credential_create",
          { pluginId, authType, config },
        );
        return result.credentialId;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential create error:`, error);
        throw error;
      }
    },

    async update(
      id: CredentialId,
      config: Record<string, unknown>,
    ): Promise<void> {
      try {
        await invoke("plugin_credential_update", {
          pluginId,
          credentialId: id,
          config,
        });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential update error:`, error);
        throw error;
      }
    },

    async delete(id: CredentialId): Promise<void> {
      try {
        await invoke("plugin_credential_delete", {
          pluginId,
          credentialId: id,
        });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential delete error:`, error);
        throw error;
      }
    },

    async validate(
      id: CredentialId,
    ): Promise<{ valid: boolean; message?: string }> {
      try {
        const result = await invoke<{ valid: boolean; message?: string }>(
          "plugin_credential_validate",
          { pluginId, credentialId: id },
        );
        return result;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential validate error:`, error);
        throw error;
      }
    },

    async refresh(id: CredentialId): Promise<void> {
      try {
        await invoke("plugin_credential_refresh", {
          pluginId,
          credentialId: id,
        });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Credential refresh error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// Plugin Config API 实现
// ============================================================================

function createPluginConfigApi(pluginId: PluginId): PluginConfigApi {
  return {
    async get<T = Record<string, unknown>>(): Promise<T> {
      try {
        const result = await invoke<{ config: T }>("plugin_config_get", {
          pluginId,
        });
        return result.config;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Config get error:`, error);
        throw error;
      }
    },

    async set(config: Record<string, unknown>): Promise<void> {
      try {
        await invoke("plugin_config_set", { pluginId, config });
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Config set error:`, error);
        throw error;
      }
    },

    async getValue<T = unknown>(key: string): Promise<T | null> {
      try {
        const config = await this.get();
        return (config as Record<string, unknown>)[key] as T | null;
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Config getValue error:`, error);
        throw error;
      }
    },

    async setValue(key: string, value: unknown): Promise<void> {
      try {
        const config = await this.get();
        (config as Record<string, unknown>)[key] = value;
        await this.set(config);
      } catch (error) {
        console.error(`[Plugin ${pluginId}] Config setValue error:`, error);
        throw error;
      }
    },
  };
}

// ============================================================================
// SDK 工厂函数
// ============================================================================

/**
 * 创建插件 SDK 实例
 *
 * @param pluginId 插件 ID
 * @returns ProxyCast Plugin SDK 实例
 *
 * @example
 * ```tsx
 * import { createPluginSDK } from '@proxycast/plugin-sdk';
 *
 * function MyPluginUI({ pluginId }: { pluginId: string }) {
 *   const sdk = createPluginSDK(pluginId);
 *
 *   useEffect(() => {
 *     sdk.credential.list().then(console.log);
 *   }, []);
 *
 *   return <div>My Plugin</div>;
 * }
 * ```
 */
export function createPluginSDK(pluginId: PluginId): ProxyCastPluginSDK {
  return {
    pluginId,
    database: createDatabaseApi(pluginId),
    http: createHttpApi(pluginId),
    crypto: createCryptoApi(pluginId),
    notification: createNotificationApi(pluginId),
    events: createEventsApi(pluginId),
    storage: createStorageApi(pluginId),
    credential: createCredentialApi(pluginId),
    config: createPluginConfigApi(pluginId),
  };
}

// ============================================================================
// SDK 缓存
// ============================================================================

const sdkCache = new Map<PluginId, ProxyCastPluginSDK>();

/**
 * 获取或创建插件 SDK 实例（带缓存）
 *
 * @param pluginId 插件 ID
 * @returns ProxyCast Plugin SDK 实例
 */
export function getPluginSDK(pluginId: PluginId): ProxyCastPluginSDK {
  let sdk = sdkCache.get(pluginId);
  if (!sdk) {
    sdk = createPluginSDK(pluginId);
    sdkCache.set(pluginId, sdk);
  }
  return sdk;
}

/**
 * 清除 SDK 缓存
 *
 * @param pluginId 可选的插件 ID，如果不提供则清除所有缓存
 */
export function clearSDKCache(pluginId?: PluginId): void {
  if (pluginId) {
    sdkCache.delete(pluginId);
  } else {
    sdkCache.clear();
  }
}

// ============================================================================
// 全局事件订阅
// ============================================================================

/**
 * 订阅通知事件（用于全局通知显示）
 */
export function subscribeNotifications(
  callback: (notification: {
    level: "success" | "error" | "info" | "warning";
    message: string;
    pluginId: string;
  }) => void,
): Unsubscribe {
  return globalEventBus.on("notification", callback);
}

/**
 * 获取全局事件总线（用于高级场景）
 */
export function getGlobalEventBus(): PluginEventBus {
  return globalEventBus;
}
