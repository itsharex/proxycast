/**
 * @file 组件注册表
 * @description 管理插件 UI 可用的组件类型，实现组件白名单机制
 * @module lib/plugin-ui/ComponentRegistry
 */

import type {
  ComponentRenderer,
  ComponentRegistryEntry,
  ComponentTypeName,
} from "./types";

/**
 * 组件注册表
 * 管理所有可用的 UI 组件，插件只能使用已注册的组件
 */
export class ComponentRegistry {
  private registry: Map<string, ComponentRegistryEntry> = new Map();

  /**
   * 注册组件
   * @param typeName - 组件类型名称
   * @param renderer - 组件渲染器
   * @param schema - 可选的组件属性 schema
   */
  register(
    typeName: ComponentTypeName | string,
    renderer: ComponentRenderer,
    schema?: Record<string, unknown>,
  ): void {
    if (!/^[a-zA-Z][a-zA-Z0-9]*$/.test(typeName)) {
      throw new Error(
        `[ComponentRegistry] 无效的组件名称 '${typeName}'，必须以字母开头且只包含字母数字`,
      );
    }

    if (this.registry.has(typeName)) {
      console.warn(`[ComponentRegistry] 组件 '${typeName}' 已存在，将被覆盖`);
    }

    this.registry.set(typeName, { renderer, schema });
  }

  /**
   * 获取组件渲染器
   * @param typeName - 组件类型名称
   * @returns 组件渲染器，如果未注册则返回 undefined
   */
  get(typeName: string): ComponentRenderer | undefined {
    return this.registry.get(typeName)?.renderer;
  }

  /**
   * 获取组件 schema
   * @param typeName - 组件类型名称
   * @returns 组件 schema
   */
  getSchema(typeName: string): Record<string, unknown> | undefined {
    return this.registry.get(typeName)?.schema;
  }

  /**
   * 检查组件是否已注册
   * @param typeName - 组件类型名称
   */
  has(typeName: string): boolean {
    return this.registry.has(typeName);
  }

  /**
   * 获取所有已注册的组件类型名称
   */
  getRegisteredTypes(): string[] {
    return Array.from(this.registry.keys());
  }

  /**
   * 批量注册组件
   * @param components - 组件映射
   */
  registerAll(components: Record<string, ComponentRenderer>): void {
    for (const [typeName, renderer] of Object.entries(components)) {
      this.register(typeName, renderer);
    }
  }

  /**
   * 注销组件
   * @param typeName - 组件类型名称
   */
  unregister(typeName: string): boolean {
    return this.registry.delete(typeName);
  }

  /**
   * 清空注册表
   */
  clear(): void {
    this.registry.clear();
  }
}

/**
 * 全局组件注册表实例
 */
export const componentRegistry = new ComponentRegistry();
