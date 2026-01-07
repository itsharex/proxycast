/**
 * @file registry.ts
 * @description 块注册表
 * @module lib/blocks/registry
 *
 * 管理块类型的注册和查找。
 */

import type React from "react";
import type {
  BlockType,
  BlockRegistryEntry,
  BlockComponentProps,
} from "./types";

/** 块注册表 */
class BlockRegistry {
  private entries: Map<BlockType, BlockRegistryEntry> = new Map();

  /**
   * 注册块类型
   */
  register(entry: BlockRegistryEntry): void {
    this.entries.set(entry.type, entry);
  }

  /**
   * 获取块类型配置
   */
  get(type: BlockType): BlockRegistryEntry | undefined {
    return this.entries.get(type);
  }

  /**
   * 获取所有可创建的块类型
   */
  getCreatableTypes(): BlockRegistryEntry[] {
    return Array.from(this.entries.values()).filter(
      (entry) => entry.canCreate !== false,
    );
  }

  /**
   * 获取所有块类型
   */
  getAll(): BlockRegistryEntry[] {
    return Array.from(this.entries.values());
  }

  /**
   * 检查块类型是否已注册
   */
  has(type: BlockType): boolean {
    return this.entries.has(type);
  }

  /**
   * 获取块组件
   */
  getComponent(
    type: BlockType,
  ): React.ComponentType<BlockComponentProps> | undefined {
    return this.entries.get(type)?.component;
  }

  /**
   * 获取块图标
   */
  getIcon(type: BlockType): string {
    return this.entries.get(type)?.icon ?? "square";
  }

  /**
   * 获取块显示名称
   */
  getDisplayName(type: BlockType): string {
    return this.entries.get(type)?.displayName ?? type;
  }
}

/** 全局块注册表实例 */
export const blockRegistry = new BlockRegistry();

/** 注册块类型的便捷函数 */
export function registerBlockType(entry: BlockRegistryEntry): void {
  blockRegistry.register(entry);
}

/** 获取块类型配置的便捷函数 */
export function getBlockType(type: BlockType): BlockRegistryEntry | undefined {
  return blockRegistry.get(type);
}
