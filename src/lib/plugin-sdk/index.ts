/**
 * @file ProxyCast Plugin SDK 入口
 * @description 提供给 OAuth Provider 插件 UI 使用的 SDK
 * @module lib/plugin-sdk
 *
 * @example
 * ```tsx
 * import { createPluginSDK, type ProxyCastPluginSDK } from '@/lib/plugin-sdk';
 *
 * function MyPluginUI({ pluginId }: { pluginId: string }) {
 *   const sdk = createPluginSDK(pluginId);
 *
 *   const loadCredentials = async () => {
 *     const credentials = await sdk.credential.list();
 *     console.log(credentials);
 *   };
 *
 *   return <button onClick={loadCredentials}>Load</button>;
 * }
 * ```
 */

// 类型导出
export type {
  // 基础类型
  PluginId,
  CredentialId,

  // API 类型
  DatabaseApi,
  HttpApi,
  CryptoApi,
  NotificationApi,
  EventsApi,
  StorageApi,
  CredentialApi,
  PluginConfigApi,

  // 数据类型
  QueryResult,
  ExecuteResult,
  HttpRequestOptions,
  HttpResponse,
  EventCallback,
  Unsubscribe,
  CredentialInfo,

  // 主 SDK 类型
  ProxyCastPluginSDK,

  // 组件类型
  PluginUIProps,
  PluginMetadata,
  PluginEntry,
} from "./types";

// SDK 实现导出
export {
  createPluginSDK,
  getPluginSDK,
  clearSDKCache,
  subscribeNotifications,
  getGlobalEventBus,
} from "./sdk";

// Hook 导出
export { usePluginSDK } from "./usePluginSDK";
