/**
 * @file ProxyCast Plugin SDK 类型定义
 * @description 提供给 OAuth Provider 插件 UI 使用的 SDK 类型
 * @module lib/plugin-sdk/types
 */

import type React from "react";

// ============================================================================
// 基础类型
// ============================================================================

/** 插件 ID */
export type PluginId = string;

/** 凭证 ID */
export type CredentialId = string;

// ============================================================================
// 数据库操作
// ============================================================================

/** 查询结果 */
export interface QueryResult<T = Record<string, unknown>> {
  /** 列名 */
  columns: string[];
  /** 行数据 */
  rows: T[];
}

/** 执行结果 */
export interface ExecuteResult {
  /** 影响的行数 */
  affected: number;
}

/** 数据库操作接口 */
export interface DatabaseApi {
  /**
   * 执行 SELECT 查询
   * @param sql SQL 查询语句（仅支持 SELECT）
   * @param params 参数数组
   */
  query<T = Record<string, unknown>>(
    sql: string,
    params?: unknown[],
  ): Promise<QueryResult<T>>;

  /**
   * 执行 INSERT/UPDATE/DELETE 操作
   * @param sql SQL 语句
   * @param params 参数数组
   */
  execute(sql: string, params?: unknown[]): Promise<ExecuteResult>;
}

// ============================================================================
// HTTP 操作
// ============================================================================

/** HTTP 请求选项 */
export interface HttpRequestOptions {
  /** HTTP 方法 */
  method?: "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD";
  /** 请求头 */
  headers?: Record<string, string>;
  /** 请求体 */
  body?: string;
  /** 超时（毫秒） */
  timeoutMs?: number;
}

/** HTTP 响应 */
export interface HttpResponse {
  /** 状态码 */
  status: number;
  /** 响应头 */
  headers: Record<string, string>;
  /** 响应体 */
  body: string;
}

/** HTTP 操作接口 */
export interface HttpApi {
  /**
   * 发送 HTTP 请求
   * @param url 请求 URL
   * @param options 请求选项
   */
  request(url: string, options?: HttpRequestOptions): Promise<HttpResponse>;
}

// ============================================================================
// 加密操作
// ============================================================================

/** 加密操作接口 */
export interface CryptoApi {
  /**
   * 加密数据
   * @param data 要加密的数据
   * @returns 加密后的数据
   */
  encrypt(data: string): Promise<string>;

  /**
   * 解密数据
   * @param data 要解密的数据
   * @returns 解密后的数据
   */
  decrypt(data: string): Promise<string>;
}

// ============================================================================
// 通知操作
// ============================================================================

/** 通知操作接口 */
export interface NotificationApi {
  /**
   * 显示成功通知
   * @param message 通知消息
   */
  success(message: string): void;

  /**
   * 显示错误通知
   * @param message 通知消息
   */
  error(message: string): void;

  /**
   * 显示信息通知
   * @param message 通知消息
   */
  info(message: string): void;

  /**
   * 显示警告通知
   * @param message 通知消息
   */
  warning(message: string): void;
}

// ============================================================================
// 事件操作
// ============================================================================

/** 事件回调类型 */
export type EventCallback<T = unknown> = (data: T) => void;

/** 取消订阅函数 */
export type Unsubscribe = () => void;

/** 事件操作接口 */
export interface EventsApi {
  /**
   * 发布事件
   * @param event 事件名称
   * @param data 事件数据
   */
  emit(event: string, data?: unknown): void;

  /**
   * 订阅事件
   * @param event 事件名称
   * @param callback 回调函数
   * @returns 取消订阅函数
   */
  on<T = unknown>(event: string, callback: EventCallback<T>): Unsubscribe;

  /**
   * 一次性订阅事件
   * @param event 事件名称
   * @param callback 回调函数
   */
  once<T = unknown>(event: string, callback: EventCallback<T>): void;
}

// ============================================================================
// 存储操作
// ============================================================================

/** 存储操作接口 */
export interface StorageApi {
  /**
   * 获取存储的值
   * @param key 键名
   */
  get(key: string): Promise<string | null>;

  /**
   * 设置存储的值
   * @param key 键名
   * @param value 值
   */
  set(key: string, value: string): Promise<void>;

  /**
   * 删除存储的值
   * @param key 键名
   */
  delete(key: string): Promise<void>;

  /**
   * 获取所有键
   */
  keys(): Promise<string[]>;
}

// ============================================================================
// 凭证操作
// ============================================================================

/** 凭证基本信息 */
export interface CredentialInfo {
  /** 凭证 ID */
  id: string;
  /** 插件 ID */
  pluginId: string;
  /** 认证类型 */
  authType: string;
  /** 显示名称 */
  displayName: string;
  /** 状态 */
  status: "active" | "inactive" | "expired" | "error";
  /** 创建时间 */
  createdAt: string;
  /** 更新时间 */
  updatedAt: string;
  /** 最后使用时间 */
  lastUsedAt?: string;
  /** 额外配置 */
  config: Record<string, unknown>;
}

/** 凭证操作接口 */
export interface CredentialApi {
  /**
   * 获取插件的所有凭证
   */
  list(): Promise<CredentialInfo[]>;

  /**
   * 获取单个凭证
   * @param id 凭证 ID
   */
  get(id: CredentialId): Promise<CredentialInfo | null>;

  /**
   * 创建新凭证
   * @param authType 认证类型
   * @param config 凭证配置
   */
  create(
    authType: string,
    config: Record<string, unknown>,
  ): Promise<CredentialId>;

  /**
   * 更新凭证
   * @param id 凭证 ID
   * @param config 新配置
   */
  update(id: CredentialId, config: Record<string, unknown>): Promise<void>;

  /**
   * 删除凭证
   * @param id 凭证 ID
   */
  delete(id: CredentialId): Promise<void>;

  /**
   * 验证凭证
   * @param id 凭证 ID
   */
  validate(id: CredentialId): Promise<{ valid: boolean; message?: string }>;

  /**
   * 刷新凭证（用于 OAuth Token 刷新）
   * @param id 凭证 ID
   */
  refresh(id: CredentialId): Promise<void>;
}

// ============================================================================
// 插件配置
// ============================================================================

/** 插件配置接口 */
export interface PluginConfigApi {
  /**
   * 获取插件配置
   */
  get<T = Record<string, unknown>>(): Promise<T>;

  /**
   * 更新插件配置
   * @param config 新配置
   */
  set(config: Record<string, unknown>): Promise<void>;

  /**
   * 获取配置中的某个值
   * @param key 配置键
   */
  getValue<T = unknown>(key: string): Promise<T | null>;

  /**
   * 设置配置中的某个值
   * @param key 配置键
   * @param value 值
   */
  setValue(key: string, value: unknown): Promise<void>;
}

// ============================================================================
// 主 SDK 接口
// ============================================================================

/**
 * ProxyCast 插件 SDK
 *
 * 提供给 OAuth Provider 插件 UI 使用的完整接口
 */
export interface ProxyCastPluginSDK {
  /** 插件 ID */
  readonly pluginId: PluginId;

  /** 数据库操作 */
  readonly database: DatabaseApi;

  /** HTTP 操作 */
  readonly http: HttpApi;

  /** 加密操作 */
  readonly crypto: CryptoApi;

  /** 通知操作 */
  readonly notification: NotificationApi;

  /** 事件操作 */
  readonly events: EventsApi;

  /** 存储操作 */
  readonly storage: StorageApi;

  /** 凭证操作 */
  readonly credential: CredentialApi;

  /** 插件配置操作 */
  readonly config: PluginConfigApi;
}

// ============================================================================
// 插件组件 Props
// ============================================================================

/**
 * 插件 UI 组件 Props
 *
 * 每个插件的根组件都会接收这些 props
 */
export interface PluginUIProps {
  /** ProxyCast SDK 实例 */
  sdk: ProxyCastPluginSDK;
  /** 插件 ID */
  pluginId: PluginId;
}

// ============================================================================
// 插件注册
// ============================================================================

/** 插件元数据 */
export interface PluginMetadata {
  /** 插件 ID */
  id: PluginId;
  /** 显示名称 */
  displayName: string;
  /** 版本 */
  version: string;
  /** 描述 */
  description?: string;
  /** 作者 */
  author?: string;
  /** 图标 */
  icon?: string;
}

/** 插件入口点 */
export interface PluginEntry {
  /** 插件元数据 */
  metadata: PluginMetadata;
  /** 主组件 */
  component: React.ComponentType<PluginUIProps>;
  /** 设置组件（可选） */
  settingsComponent?: React.ComponentType<PluginUIProps>;
}
