/**
 * @file types.ts
 * @description 块系统类型定义
 * @module lib/blocks/types
 *
 * 参考 waveterm 的块系统设计，定义块的核心类型和接口。
 */

import type React from "react";

/** 块类型枚举 */
export type BlockType =
  | "terminal"
  | "preview"
  | "web"
  | "ai"
  | "markdown"
  | "image";

/** 块状态 */
export type BlockStatus = "idle" | "loading" | "active" | "error";

/** 块元数据 */
export interface BlockMeta {
  /** 块标题 */
  title?: string;
  /** 块图标 */
  icon?: string;
  /** 自定义数据 */
  data?: Record<string, unknown>;
}

/** 块配置 */
export interface BlockConfig {
  /** 块类型 */
  type: BlockType;
  /** 块元数据 */
  meta?: BlockMeta;
  /** 终端会话 ID（仅终端块） */
  sessionId?: string;
  /** 文件路径（仅预览块） */
  filePath?: string;
  /** URL（仅 Web 块） */
  url?: string;
}

/** 块实例 */
export interface Block {
  /** 块唯一 ID */
  id: string;
  /** 块类型 */
  type: BlockType;
  /** 块状态 */
  status: BlockStatus;
  /** 块元数据 */
  meta: BlockMeta;
  /** 块配置 */
  config: BlockConfig;
  /** 创建时间 */
  createdAt: number;
  /** 更新时间 */
  updatedAt: number;
}

/** 块视图模型接口 */
export interface BlockViewModel {
  /** 块 ID */
  blockId: string;
  /** 是否聚焦 */
  isFocused: boolean;
  /** 是否最大化 */
  isMagnified: boolean;
  /** 聚焦块 */
  focus: () => void;
  /** 切换最大化 */
  toggleMagnify: () => void;
  /** 关闭块 */
  close: () => void;
}

/** 块组件属性 */
export interface BlockComponentProps {
  /** 块实例 */
  block: Block;
  /** 视图模型 */
  viewModel: BlockViewModel;
  /** 是否可见 */
  visible?: boolean;
}

/** 块注册表项 */
export interface BlockRegistryEntry {
  /** 块类型 */
  type: BlockType;
  /** 显示名称 */
  displayName: string;
  /** 图标名称 */
  icon: string;
  /** 组件 */
  component: React.ComponentType<BlockComponentProps>;
  /** 是否可创建 */
  canCreate?: boolean;
  /** 创建块的默认配置 */
  defaultConfig?: Partial<BlockConfig>;
}

/** 面板方向 */
export type PanelDirection = "horizontal" | "vertical";

/** 面板节点类型 */
export type PanelNodeType = "block" | "group";

/** 面板节点 */
export interface PanelNode {
  /** 节点 ID */
  id: string;
  /** 节点类型 */
  type: PanelNodeType;
  /** 块 ID（仅 block 类型） */
  blockId?: string;
  /** 子节点（仅 group 类型） */
  children?: PanelNode[];
  /** 分割方向（仅 group 类型） */
  direction?: PanelDirection;
  /** 面板大小百分比 */
  size?: number;
}

/** 标签页 */
export interface Tab {
  /** 标签页 ID */
  id: string;
  /** 标签页标题 */
  title: string;
  /** 根面板节点 */
  rootNode: PanelNode;
  /** 活动块 ID */
  activeBlockId?: string;
  /** 是否固定 */
  isPinned?: boolean;
}

/** 工作区 */
export interface Workspace {
  /** 工作区 ID */
  id: string;
  /** 工作区名称 */
  name: string;
  /** 标签页列表 */
  tabs: Tab[];
  /** 活动标签页 ID */
  activeTabId?: string;
  /** 创建时间 */
  createdAt: number;
  /** 更新时间 */
  updatedAt: number;
}

/** 块操作类型 */
export type BlockAction =
  | { type: "CREATE"; payload: BlockConfig }
  | { type: "UPDATE"; payload: { id: string; updates: Partial<Block> } }
  | { type: "DELETE"; payload: { id: string } }
  | { type: "FOCUS"; payload: { id: string } }
  | { type: "MAGNIFY"; payload: { id: string } }
  | {
      type: "SPLIT";
      payload: {
        blockId: string;
        direction: PanelDirection;
        newBlockConfig: BlockConfig;
      };
    };

/** 布局操作类型 */
export type LayoutAction =
  | { type: "SPLIT_HORIZONTAL"; payload: { nodeId: string } }
  | { type: "SPLIT_VERTICAL"; payload: { nodeId: string } }
  | { type: "CLOSE_PANEL"; payload: { nodeId: string } }
  | { type: "RESIZE_PANEL"; payload: { nodeId: string; size: number } }
  | {
      type: "MOVE_PANEL";
      payload: {
        sourceId: string;
        targetId: string;
        position: "before" | "after";
      };
    };
