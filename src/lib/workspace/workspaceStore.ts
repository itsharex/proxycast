/**
 * @file workspaceStore.ts
 * @description 工作区状态管理
 * @module lib/workspace/workspaceStore
 */

import { atom } from "jotai";
import type { Workspace, Tab, PanelNode } from "@/lib/blocks/types";
import { generateId } from "@/lib/blocks/blockStore";
import type { WorkspaceConfig, TabConfig } from "./types";

// ============================================================================
// 工具函数
// ============================================================================

/** 创建标签页 */
export function createTab(config: TabConfig): { tab: Tab; blockId?: string } {
  const tabId = generateId();
  let blockId: string | undefined;
  let rootNode: PanelNode;

  if (config.blockConfig) {
    blockId = generateId();
    rootNode = {
      id: generateId(),
      type: "block",
      blockId,
      size: 100,
    };
  } else {
    rootNode = {
      id: generateId(),
      type: "group",
      children: [],
      direction: "horizontal",
    };
  }

  const tab: Tab = {
    id: tabId,
    title: config.title,
    rootNode,
    activeBlockId: blockId,
    isPinned: config.isPinned,
  };

  return { tab, blockId };
}

/** 创建工作区 */
export function createWorkspace(config: WorkspaceConfig): Workspace {
  const now = Date.now();
  const workspace: Workspace = {
    id: generateId(),
    name: config.name,
    tabs: [],
    createdAt: now,
    updatedAt: now,
  };

  return workspace;
}

// ============================================================================
// 原子状态
// ============================================================================

/** 所有工作区 */
export const workspacesAtom = atom<Map<string, Workspace>>(new Map());

/** 当前活动工作区 ID */
export const activeWorkspaceIdAtom = atom<string | null>(null);

// ============================================================================
// 派生原子
// ============================================================================

/** 获取当前活动工作区 */
export const activeWorkspaceAtom = atom((get) => {
  const activeId = get(activeWorkspaceIdAtom);
  if (!activeId) return null;
  return get(workspacesAtom).get(activeId) ?? null;
});

/** 获取所有工作区列表 */
export const workspaceListAtom = atom((get) => {
  return Array.from(get(workspacesAtom).values());
});

/** 获取当前工作区的标签页列表 */
export const currentTabsAtom = atom((get) => {
  const workspace = get(activeWorkspaceAtom);
  return workspace?.tabs ?? [];
});

/** 获取当前活动标签页 */
export const currentActiveTabAtom = atom((get) => {
  const workspace = get(activeWorkspaceAtom);
  if (!workspace?.activeTabId) return null;
  return workspace.tabs.find((t) => t.id === workspace.activeTabId) ?? null;
});

// ============================================================================
// 操作原子
// ============================================================================

/** 添加工作区 */
export const addWorkspaceAtom = atom(
  null,
  (get, set, config: WorkspaceConfig) => {
    const workspace = createWorkspace(config);
    const workspaces = new Map(get(workspacesAtom));
    workspaces.set(workspace.id, workspace);
    set(workspacesAtom, workspaces);
    set(activeWorkspaceIdAtom, workspace.id);
    return workspace;
  },
);

/** 删除工作区 */
export const removeWorkspaceAtom = atom(
  null,
  (get, set, workspaceId: string) => {
    const workspaces = new Map(get(workspacesAtom));
    workspaces.delete(workspaceId);
    set(workspacesAtom, workspaces);

    // 如果删除的是当前工作区，切换到其他工作区
    if (get(activeWorkspaceIdAtom) === workspaceId) {
      const remaining = Array.from(workspaces.values());
      set(activeWorkspaceIdAtom, remaining.length > 0 ? remaining[0].id : null);
    }
  },
);

/** 切换工作区 */
export const switchWorkspaceAtom = atom(
  null,
  (get, set, workspaceId: string) => {
    set(activeWorkspaceIdAtom, workspaceId);
  },
);

/** 添加标签页到当前工作区 */
export const addTabToWorkspaceAtom = atom(
  null,
  (get, set, config: TabConfig) => {
    const activeId = get(activeWorkspaceIdAtom);
    if (!activeId) return null;

    const workspaces = new Map(get(workspacesAtom));
    const workspace = workspaces.get(activeId);
    if (!workspace) return null;

    const { tab } = createTab(config);
    const updatedWorkspace: Workspace = {
      ...workspace,
      tabs: [...workspace.tabs, tab],
      activeTabId: tab.id,
      updatedAt: Date.now(),
    };

    workspaces.set(activeId, updatedWorkspace);
    set(workspacesAtom, workspaces);

    return tab;
  },
);

/** 删除标签页 */
export const removeTabFromWorkspaceAtom = atom(
  null,
  (get, set, tabId: string) => {
    const activeId = get(activeWorkspaceIdAtom);
    if (!activeId) return;

    const workspaces = new Map(get(workspacesAtom));
    const workspace = workspaces.get(activeId);
    if (!workspace) return;

    const tabIndex = workspace.tabs.findIndex((t) => t.id === tabId);
    if (tabIndex === -1) return;

    const newTabs = workspace.tabs.filter((t) => t.id !== tabId);
    let newActiveTabId = workspace.activeTabId;

    // 如果删除的是当前标签页，切换到相邻标签页
    if (workspace.activeTabId === tabId) {
      if (newTabs.length > 0) {
        const newIndex = Math.min(tabIndex, newTabs.length - 1);
        newActiveTabId = newTabs[newIndex].id;
      } else {
        newActiveTabId = undefined;
      }
    }

    const updatedWorkspace: Workspace = {
      ...workspace,
      tabs: newTabs,
      activeTabId: newActiveTabId,
      updatedAt: Date.now(),
    };

    workspaces.set(activeId, updatedWorkspace);
    set(workspacesAtom, workspaces);
  },
);

/** 切换标签页 */
export const switchTabAtom = atom(null, (get, set, tabId: string) => {
  const activeId = get(activeWorkspaceIdAtom);
  if (!activeId) return;

  const workspaces = new Map(get(workspacesAtom));
  const workspace = workspaces.get(activeId);
  if (!workspace) return;

  const updatedWorkspace: Workspace = {
    ...workspace,
    activeTabId: tabId,
    updatedAt: Date.now(),
  };

  workspaces.set(activeId, updatedWorkspace);
  set(workspacesAtom, workspaces);
});

/** 重命名标签页 */
export const renameTabAtom = atom(
  null,
  (get, set, { tabId, title }: { tabId: string; title: string }) => {
    const activeId = get(activeWorkspaceIdAtom);
    if (!activeId) return;

    const workspaces = new Map(get(workspacesAtom));
    const workspace = workspaces.get(activeId);
    if (!workspace) return;

    const updatedTabs = workspace.tabs.map((tab) =>
      tab.id === tabId ? { ...tab, title } : tab,
    );

    const updatedWorkspace: Workspace = {
      ...workspace,
      tabs: updatedTabs,
      updatedAt: Date.now(),
    };

    workspaces.set(activeId, updatedWorkspace);
    set(workspacesAtom, workspaces);
  },
);

/** 移动标签页 */
export const moveTabAtom = atom(
  null,
  (get, set, { tabId, newIndex }: { tabId: string; newIndex: number }) => {
    const activeId = get(activeWorkspaceIdAtom);
    if (!activeId) return;

    const workspaces = new Map(get(workspacesAtom));
    const workspace = workspaces.get(activeId);
    if (!workspace) return;

    const currentIndex = workspace.tabs.findIndex((t) => t.id === tabId);
    if (currentIndex === -1 || currentIndex === newIndex) return;

    const newTabs = [...workspace.tabs];
    const [removed] = newTabs.splice(currentIndex, 1);
    newTabs.splice(newIndex, 0, removed);

    const updatedWorkspace: Workspace = {
      ...workspace,
      tabs: newTabs,
      updatedAt: Date.now(),
    };

    workspaces.set(activeId, updatedWorkspace);
    set(workspacesAtom, workspaces);
  },
);
