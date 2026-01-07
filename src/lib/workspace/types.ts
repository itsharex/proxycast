/**
 * @file types.ts
 * @description 工作区类型定义
 * @module lib/workspace/types
 */

import type { Workspace, BlockConfig } from "@/lib/blocks/types";

/** 工作区配置 */
export interface WorkspaceConfig {
  /** 工作区名称 */
  name: string;
  /** 初始标签页配置 */
  initialTabs?: TabConfig[];
}

/** 标签页配置 */
export interface TabConfig {
  /** 标签页标题 */
  title: string;
  /** 初始块配置 */
  blockConfig?: BlockConfig;
  /** 是否固定 */
  isPinned?: boolean;
}

/** 工作区操作类型 */
export type WorkspaceAction =
  | { type: "CREATE_WORKSPACE"; payload: WorkspaceConfig }
  | { type: "DELETE_WORKSPACE"; payload: { id: string } }
  | { type: "RENAME_WORKSPACE"; payload: { id: string; name: string } }
  | { type: "SWITCH_WORKSPACE"; payload: { id: string } }
  | { type: "CREATE_TAB"; payload: { workspaceId: string; config: TabConfig } }
  | { type: "DELETE_TAB"; payload: { workspaceId: string; tabId: string } }
  | { type: "SWITCH_TAB"; payload: { workspaceId: string; tabId: string } }
  | {
      type: "MOVE_TAB";
      payload: { workspaceId: string; tabId: string; newIndex: number };
    }
  | {
      type: "PIN_TAB";
      payload: { workspaceId: string; tabId: string; isPinned: boolean };
    };

/** 工作区状态 */
export interface WorkspaceState {
  /** 所有工作区 */
  workspaces: Map<string, Workspace>;
  /** 当前活动工作区 ID */
  activeWorkspaceId: string | null;
}

/** 工作区存储数据 */
export interface WorkspaceStorageData {
  workspaces: Array<{
    id: string;
    name: string;
    tabs: Array<{
      id: string;
      title: string;
      isPinned?: boolean;
    }>;
    activeTabId?: string;
    createdAt: number;
    updatedAt: number;
  }>;
  activeWorkspaceId: string | null;
}
