/**
 * @file Safe Tauri Invoke 封装
 * @description 提供安全的 Tauri invoke 调用，支持三层 fallback：
 *   1. Tauri IPC (生产环境或 Tauri webview)
 *   2. HTTP Bridge (开发模式，浏览器 + Tauri 后端)
 *   3. Mock (仅测试/非浏览器调试场景)
 *
 * @module dev-bridge/safeInvoke
 */

import { invoke as baseInvoke } from "@tauri-apps/api/core";
import { listen as baseListen, emit as baseEmit } from "@tauri-apps/api/event";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  invokeViaHttp,
  isDevBridgeAvailable,
  normalizeDevBridgeError,
} from "./http-client";
import { shouldPreferMockInBrowser } from "./mockPriorityCommands";

export interface InvokeErrorBufferEntry {
  timestamp: string;
  command: string;
  transport: "tauri-ipc" | "tauri-legacy" | "http-bridge" | "fallback-invoke";
  error: string;
  args_preview?: Record<string, unknown>;
}

const INVOKE_ERROR_BUFFER_KEY = "proxycast_invoke_error_buffer_v1";
const INVOKE_ERROR_BUFFER_LIMIT = 120;
const INVOKE_ERROR_TEXT_LIMIT = 800;

const SECRET_PATTERNS: Array<[RegExp, string]> = [
  [/\bBearer\s+[A-Za-z0-9._-]+\b/gi, "Bearer ***"],
  [/\bapi[_-]?key\s*[:=]\s*["']?[A-Za-z0-9._-]+["']?/gi, "api_key=***"],
  [/\baccess[_-]?token\s*[:=]\s*["']?[A-Za-z0-9._-]+["']?/gi, "access_token=***"],
  [/\brefresh[_-]?token\s*[:=]\s*["']?[A-Za-z0-9._-]+["']?/gi, "refresh_token=***"],
  [/\btoken\s*[:=]\s*["']?[A-Za-z0-9._-]{10,}["']?/gi, "token=***"],
  [/\bsk-[A-Za-z0-9]{12,}\b/g, "sk-***"],
];

function sanitizeText(input: string): string {
  let sanitized = input;
  for (const [pattern, replacement] of SECRET_PATTERNS) {
    sanitized = sanitized.replace(pattern, replacement);
  }
  return sanitized;
}

function sanitizeValue(value: unknown, depth = 0): unknown {
  if (depth > 4) {
    return "[depth_limited]";
  }
  if (value === null || value === undefined) {
    return value;
  }
  if (typeof value === "string") {
    return sanitizeText(value);
  }
  if (typeof value === "number" || typeof value === "boolean") {
    return value;
  }
  if (Array.isArray(value)) {
    return value.map((item) => sanitizeValue(item, depth + 1));
  }
  if (typeof value === "object") {
    const output: Record<string, unknown> = {};
    for (const [key, nestedValue] of Object.entries(value)) {
      output[key] = sanitizeValue(nestedValue, depth + 1);
    }
    return output;
  }
  return sanitizeText(String(value));
}

function toErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    const core = error.message || error.name || "Unknown error";
    return sanitizeText(core).slice(0, INVOKE_ERROR_TEXT_LIMIT);
  }
  return sanitizeText(String(error || "Unknown error")).slice(
    0,
    INVOKE_ERROR_TEXT_LIMIT,
  );
}

function readInvokeErrorBuffer(): InvokeErrorBufferEntry[] {
  if (typeof window === "undefined") {
    return [];
  }
  try {
    const raw = window.localStorage.getItem(INVOKE_ERROR_BUFFER_KEY);
    if (!raw) {
      return [];
    }
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }
    return parsed
      .filter(
        (item): item is InvokeErrorBufferEntry =>
          item &&
          typeof item === "object" &&
          typeof item.timestamp === "string" &&
          typeof item.command === "string" &&
          typeof item.transport === "string" &&
          typeof item.error === "string",
      )
      .slice(-INVOKE_ERROR_BUFFER_LIMIT);
  } catch {
    return [];
  }
}

function writeInvokeErrorBuffer(items: InvokeErrorBufferEntry[]): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.setItem(
      INVOKE_ERROR_BUFFER_KEY,
      JSON.stringify(items.slice(-INVOKE_ERROR_BUFFER_LIMIT)),
    );
  } catch {
    // ignore
  }
}

function recordInvokeError(
  command: string,
  args: Record<string, unknown> | undefined,
  error: unknown,
  transport: InvokeErrorBufferEntry["transport"],
): void {
  const current = readInvokeErrorBuffer();
  const entry: InvokeErrorBufferEntry = {
    timestamp: new Date().toISOString(),
    command: sanitizeText(command),
    transport,
    error: toErrorMessage(error),
    args_preview: args
      ? (sanitizeValue(args) as Record<string, unknown>)
      : undefined,
  };
  current.push(entry);
  writeInvokeErrorBuffer(current);
}

export function getInvokeErrorBuffer(limit = 50): InvokeErrorBufferEntry[] {
  const safeLimit = Number.isFinite(limit)
    ? Math.min(200, Math.max(1, Math.floor(limit)))
    : 50;
  return readInvokeErrorBuffer().slice(-safeLimit);
}

export function clearInvokeErrorBuffer(): void {
  if (typeof window === "undefined") {
    return;
  }
  try {
    window.localStorage.removeItem(INVOKE_ERROR_BUFFER_KEY);
  } catch {
    // ignore
  }
}

/**
 * 安全的 Tauri invoke 封装
 * 支持三种模式：Tauri IPC → HTTP Bridge → Mock。
 * 在浏览器开发模式下，HTTP Bridge 失败会直接报错，不再静默回退到 mock。
 */
export async function safeInvoke<T = any>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  // 1. 优先使用 Tauri IPC (生产环境或 Tauri webview 可用时)
  if (
    typeof window !== "undefined" &&
    (window as any).__TAURI__?.core?.invoke
  ) {
    try {
      return await (window as any).__TAURI__.core.invoke(cmd, args);
    } catch (error) {
      recordInvokeError(cmd, args, error, "tauri-ipc");
      throw error;
    }
  }

  // Legacy check for older Tauri versions
  if (typeof window !== "undefined" && (window as any).__TAURI__?.invoke) {
    try {
      return await (window as any).__TAURI__.invoke(cmd, args);
    } catch (error) {
      recordInvokeError(cmd, args, error, "tauri-legacy");
      throw error;
    }
  }

  // 2. 浏览器开发模式下，部分原生/非关键命令直接优先走 mock。
  if (isDevBridgeAvailable() && shouldPreferMockInBrowser(cmd)) {
    try {
      return await baseInvoke(cmd, args);
    } catch (error) {
      recordInvokeError(cmd, args, error, "fallback-invoke");
      throw error;
    }
  }

  // 3. Dev 模式下尝试 HTTP 桥接（浏览器环境，Tauri 后端在运行）
  if (isDevBridgeAvailable()) {
    try {
      const result = await invokeViaHttp(cmd, args);
      return result as T;
    } catch (error) {
      const normalizedError = normalizeDevBridgeError(cmd, error);
      recordInvokeError(cmd, args, normalizedError, "http-bridge");

      try {
        return await baseInvoke(cmd, args);
      } catch (fallbackError) {
        recordInvokeError(cmd, args, fallbackError, "fallback-invoke");
        throw normalizedError;
      }
    }
  }

  // 4. Fallback 到 mock（Vite alias 会替换 @tauri-apps 导入）
  try {
    return await baseInvoke(cmd, args);
  } catch (error) {
    recordInvokeError(cmd, args, error, "fallback-invoke");
    throw error;
  }
}

/**
 * 安全的 Tauri listen 封装
 * 优先使用真实的 Tauri event API
 */
export async function safeListen<T = any>(
  event: string,
  handler: (event: { payload: T }) => void,
): Promise<UnlistenFn> {
  // 1. 优先使用 Tauri event API
  if (
    typeof window !== "undefined" &&
    (window as any).__TAURI__?.event?.listen
  ) {
    return (window as any).__TAURI__.event.listen(event, handler);
  }

  // 2. Fallback 到 mock（Vite alias 会替换 @tauri-apps 导入）
  return baseListen(event, handler);
}

/**
 * 安全的 Tauri emit 封装
 * 优先使用真实的 Tauri event API
 */
export async function safeEmit(
  event: string,
  payload?: unknown,
): Promise<void> {
  // 1. 优先使用 Tauri event API
  if (typeof window !== "undefined" && (window as any).__TAURI__?.event?.emit) {
    return (window as any).__TAURI__.event.emit(event, payload);
  }

  // 2. Fallback 到 mock
  return baseEmit(event, payload);
}

export default safeInvoke;
