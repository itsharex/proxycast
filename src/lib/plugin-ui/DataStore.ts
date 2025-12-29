/**
 * @file 插件 UI 数据存储
 * @description 管理 Surface 的数据模型，支持路径访问和更新
 * @module lib/plugin-ui/DataStore
 */

import type { DataEntry, DataPath } from "./types";

/**
 * 将 DataEntry 数组转换为普通对象
 */
export function dataEntriesToObject(
  entries: DataEntry[],
): Record<string, unknown> {
  const result: Record<string, unknown> = {};

  for (const entry of entries) {
    if (entry.valueString !== undefined) {
      result[entry.key] = entry.valueString;
    } else if (entry.valueNumber !== undefined) {
      result[entry.key] = entry.valueNumber;
    } else if (entry.valueBoolean !== undefined) {
      result[entry.key] = entry.valueBoolean;
    } else if (entry.valueArray !== undefined) {
      result[entry.key] = entry.valueArray.map(
        (item) => dataEntriesToObject([item])[item.key],
      );
    } else if (entry.valueMap !== undefined) {
      result[entry.key] = dataEntriesToObject(entry.valueMap);
    }
  }

  return result;
}

/**
 * 解析数据路径
 * @param path - JSONPath 格式路径，如 '/user/name' 或 'name'
 * @returns 路径段数组
 */
export function parsePath(path: DataPath): string[] {
  if (!path || path === "/") return [];

  // 移除开头的 /
  const normalized = path.startsWith("/") ? path.slice(1) : path;
  return normalized.split("/").filter(Boolean);
}

/**
 * 从数据模型中获取指定路径的值
 * @param data - 数据模型
 * @param path - 数据路径
 * @returns 路径对应的值
 */
export function getValueByPath(
  data: Record<string, unknown>,
  path: DataPath,
): unknown {
  const segments = parsePath(path);

  let current: unknown = data;
  for (const segment of segments) {
    if (current === null || current === undefined) {
      return undefined;
    }

    if (typeof current === "object") {
      if (Array.isArray(current)) {
        const index = parseInt(segment, 10);
        if (isNaN(index)) return undefined;
        current = current[index];
      } else {
        current = (current as Record<string, unknown>)[segment];
      }
    } else {
      return undefined;
    }
  }

  return current;
}

/**
 * 在数据模型中设置指定路径的值
 * @param data - 数据模型
 * @param path - 数据路径
 * @param value - 要设置的值
 * @returns 更新后的数据模型（新对象）
 */
export function setValueByPath(
  data: Record<string, unknown>,
  path: DataPath,
  value: unknown,
): Record<string, unknown> {
  const segments = parsePath(path);

  if (segments.length === 0) {
    // 替换整个数据模型
    if (typeof value === "object" && value !== null && !Array.isArray(value)) {
      return { ...(value as Record<string, unknown>) };
    }
    return data;
  }

  // 深拷贝并更新
  const result = JSON.parse(JSON.stringify(data)) as Record<string, unknown>;
  let current: Record<string, unknown> = result;

  for (let i = 0; i < segments.length - 1; i++) {
    const segment = segments[i];
    const nextSegment = segments[i + 1];
    const isNextArray = !isNaN(parseInt(nextSegment, 10));

    if (current[segment] === undefined) {
      current[segment] = isNextArray ? [] : {};
    }

    current = current[segment] as Record<string, unknown>;
  }

  const lastSegment = segments[segments.length - 1];
  current[lastSegment] = value;

  return result;
}

/**
 * 合并数据更新到现有数据模型
 * @param data - 现有数据模型
 * @param path - 更新路径
 * @param entries - 数据条目
 * @returns 更新后的数据模型
 */
export function mergeDataUpdate(
  data: Record<string, unknown>,
  path: DataPath | undefined,
  entries: DataEntry[],
): Record<string, unknown> {
  const newData = dataEntriesToObject(entries);

  if (!path || path === "/") {
    // 合并到根
    return { ...data, ...newData };
  }

  // 合并到指定路径
  const existingValue = getValueByPath(data, path);
  const mergedValue =
    typeof existingValue === "object" && existingValue !== null
      ? { ...(existingValue as Record<string, unknown>), ...newData }
      : newData;

  return setValueByPath(data, path, mergedValue);
}

/**
 * 数据存储类
 * 管理单个 Surface 的数据模型
 */
export class DataStore {
  private data: Record<string, unknown> = {};
  private listeners: Set<(data: Record<string, unknown>) => void> = new Set();

  /**
   * 获取完整数据模型
   */
  getData(): Record<string, unknown> {
    return this.data;
  }

  /**
   * 获取指定路径的值
   */
  getValue(path: DataPath): unknown {
    return getValueByPath(this.data, path);
  }

  /**
   * 设置指定路径的值
   */
  setValue(path: DataPath, value: unknown): void {
    this.data = setValueByPath(this.data, path, value);
    this.notifyListeners();
  }

  /**
   * 应用数据更新
   */
  applyUpdate(path: DataPath | undefined, entries: DataEntry[]): void {
    this.data = mergeDataUpdate(this.data, path, entries);
    this.notifyListeners();
  }

  /**
   * 重置数据模型
   */
  reset(initialData?: Record<string, unknown>): void {
    this.data = initialData ?? {};
    this.notifyListeners();
  }

  /**
   * 订阅数据变化
   */
  subscribe(listener: (data: Record<string, unknown>) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners(): void {
    for (const listener of this.listeners) {
      listener(this.data);
    }
  }
}
