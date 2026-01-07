/**
 * @file blockStore.ts
 * @description 块状态管理
 * @module lib/blocks/blockStore
 *
 * 使用 Jotai 进行原子化状态管理。
 */

import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import type {
  Block,
  BlockConfig,
  Tab,
  Workspace,
  PanelNode,
  BlockType,
} from "./types";

// ============================================================================
// 工具函数
// ============================================================================

/** 生成唯一 ID */
export function generateId(): string {
  return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/** 创建块实例 */
export function createBlock(config: BlockConfig): Block {
  const now = Date.now();
  return {
    id: generateId(),
    type: config.type,
    status: "idle",
    meta: config.meta ?? {},
    config,
    createdAt: now,
    updatedAt: now,
  };
}

/** 创建面板节点 */
export function createPanelNode(blockId: string): PanelNode {
  return {
    id: generateId(),
    type: "block",
    blockId,
    size: 100,
  };
}

/** 创建标签页 */
export function createTab(title: string, rootBlockConfig?: BlockConfig): Tab {
  const block = rootBlockConfig ? createBlock(rootBlockConfig) : null;
  return {
    id: generateId(),
    title,
    rootNode: block
      ? createPanelNode(block.id)
      : {
          id: generateId(),
          type: "group",
          children: [],
          direction: "horizontal",
        },
    activeBlockId: block?.id,
  };
}

/** 创建工作区 */
export function createWorkspace(name: string): Workspace {
  const now = Date.now();
  return {
    id: generateId(),
    name,
    tabs: [],
    createdAt: now,
    updatedAt: now,
  };
}

// ============================================================================
// 原子状态
// ============================================================================

/** 所有块的映射 */
export const blocksAtom = atom<Map<string, Block>>(new Map());

/** 所有标签页的映射 */
export const tabsAtom = atom<Map<string, Tab>>(new Map());

/** 当前活动标签页 ID */
export const activeTabIdAtom = atom<string | null>(null);

/** 当前活动块 ID */
export const activeBlockIdAtom = atom<string | null>(null);

/** 最大化的块 ID */
export const magnifiedBlockIdAtom = atom<string | null>(null);

/** 当前主题 */
export const themeAtom = atomWithStorage<string>(
  "terminal-theme",
  "tokyo-night",
);

// ============================================================================
// 派生原子
// ============================================================================

/** 获取当前活动标签页 */
export const activeTabAtom = atom((get) => {
  const activeTabId = get(activeTabIdAtom);
  if (!activeTabId) return null;
  return get(tabsAtom).get(activeTabId) ?? null;
});

/** 获取当前活动块 */
export const activeBlockAtom = atom((get) => {
  const activeBlockId = get(activeBlockIdAtom);
  if (!activeBlockId) return null;
  return get(blocksAtom).get(activeBlockId) ?? null;
});

/** 获取所有标签页列表 */
export const tabListAtom = atom((get) => {
  return Array.from(get(tabsAtom).values());
});

/** 获取所有块列表 */
export const blockListAtom = atom((get) => {
  return Array.from(get(blocksAtom).values());
});

// ============================================================================
// 操作原子
// ============================================================================

/** 添加块 */
export const addBlockAtom = atom(null, (get, set, config: BlockConfig) => {
  const block = createBlock(config);
  const blocks = new Map(get(blocksAtom));
  blocks.set(block.id, block);
  set(blocksAtom, blocks);
  return block;
});

/** 更新块 */
export const updateBlockAtom = atom(
  null,
  (get, set, { id, updates }: { id: string; updates: Partial<Block> }) => {
    const blocks = new Map(get(blocksAtom));
    const block = blocks.get(id);
    if (block) {
      blocks.set(id, { ...block, ...updates, updatedAt: Date.now() });
      set(blocksAtom, blocks);
    }
  },
);

/** 删除块 */
export const removeBlockAtom = atom(null, (get, set, id: string) => {
  const blocks = new Map(get(blocksAtom));
  blocks.delete(id);
  set(blocksAtom, blocks);

  // 如果删除的是活动块，清除活动状态
  if (get(activeBlockIdAtom) === id) {
    set(activeBlockIdAtom, null);
  }
  if (get(magnifiedBlockIdAtom) === id) {
    set(magnifiedBlockIdAtom, null);
  }
});

/** 添加标签页 */
export const addTabAtom = atom(
  null,
  (
    get,
    set,
    { title, blockConfig }: { title: string; blockConfig?: BlockConfig },
  ) => {
    // 如果有块配置，先创建块
    let block: Block | null = null;
    if (blockConfig) {
      const blocks = new Map(get(blocksAtom));
      block = createBlock(blockConfig);
      blocks.set(block.id, block);
      set(blocksAtom, blocks);
    }

    // 创建标签页
    const tab: Tab = {
      id: generateId(),
      title,
      rootNode: block
        ? createPanelNode(block.id)
        : {
            id: generateId(),
            type: "group",
            children: [],
            direction: "horizontal",
          },
      activeBlockId: block?.id,
    };

    const tabs = new Map(get(tabsAtom));
    tabs.set(tab.id, tab);
    set(tabsAtom, tabs);
    set(activeTabIdAtom, tab.id);

    if (block) {
      set(activeBlockIdAtom, block.id);
    }

    return tab;
  },
);

/** 删除标签页 */
export const removeTabAtom = atom(null, (get, set, tabId: string) => {
  const tabs = new Map(get(tabsAtom));
  const tab = tabs.get(tabId);

  if (tab) {
    // 删除标签页中的所有块
    const blocks = new Map(get(blocksAtom));
    const blockIds = collectBlockIds(tab.rootNode);
    blockIds.forEach((id) => blocks.delete(id));
    set(blocksAtom, blocks);

    // 删除标签页
    tabs.delete(tabId);
    set(tabsAtom, tabs);

    // 如果删除的是活动标签页，切换到其他标签页
    if (get(activeTabIdAtom) === tabId) {
      const remainingTabs = Array.from(tabs.values());
      set(
        activeTabIdAtom,
        remainingTabs.length > 0
          ? remainingTabs[remainingTabs.length - 1].id
          : null,
      );
    }
  }
});

/** 设置活动标签页 */
export const setActiveTabAtom = atom(null, (get, set, tabId: string) => {
  set(activeTabIdAtom, tabId);
  const tab = get(tabsAtom).get(tabId);
  if (tab?.activeBlockId) {
    set(activeBlockIdAtom, tab.activeBlockId);
  }
});

/** 设置活动块 */
export const setActiveBlockAtom = atom(null, (get, set, blockId: string) => {
  set(activeBlockIdAtom, blockId);

  // 更新标签页的活动块
  const activeTabId = get(activeTabIdAtom);
  if (activeTabId) {
    const tabs = new Map(get(tabsAtom));
    const tab = tabs.get(activeTabId);
    if (tab) {
      tabs.set(activeTabId, { ...tab, activeBlockId: blockId });
      set(tabsAtom, tabs);
    }
  }
});

/** 切换块最大化 */
export const toggleMagnifyAtom = atom(null, (get, set, blockId: string) => {
  const currentMagnified = get(magnifiedBlockIdAtom);
  set(magnifiedBlockIdAtom, currentMagnified === blockId ? null : blockId);
});

// ============================================================================
// 辅助函数
// ============================================================================

/** 收集面板节点中的所有块 ID */
function collectBlockIds(node: PanelNode): string[] {
  if (node.type === "block" && node.blockId) {
    return [node.blockId];
  }
  if (node.type === "group" && node.children) {
    return node.children.flatMap(collectBlockIds);
  }
  return [];
}

/** 在面板节点中查找块 */
export function findBlockInPanel(
  node: PanelNode,
  blockId: string,
): PanelNode | null {
  if (node.type === "block" && node.blockId === blockId) {
    return node;
  }
  if (node.type === "group" && node.children) {
    for (const child of node.children) {
      const found = findBlockInPanel(child, blockId);
      if (found) return found;
    }
  }
  return null;
}

/** 获取块类型的图标 */
export function getBlockIcon(type: BlockType): string {
  const icons: Record<BlockType, string> = {
    terminal: "terminal",
    preview: "file",
    web: "globe",
    ai: "sparkles",
    markdown: "file-text",
    image: "image",
  };
  return icons[type] ?? "square";
}

/** 获取块类型的显示名称 */
export function getBlockDisplayName(type: BlockType): string {
  const names: Record<BlockType, string> = {
    terminal: "终端",
    preview: "预览",
    web: "Web",
    ai: "AI",
    markdown: "Markdown",
    image: "图片",
  };
  return names[type] ?? type;
}
