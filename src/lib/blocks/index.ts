/**
 * @file index.ts
 * @description 块系统入口
 * @module lib/blocks
 *
 * 导出块系统的所有类型和函数，并注册内置块类型。
 */

// 导出类型
export * from "./types";

// 导出注册表
export * from "./registry";

// 导出状态管理
export * from "./blockStore";

// 注册内置块类型
import { registerBlockType } from "./registry";
import { TerminalBlock } from "@/components/blocks/TerminalBlock";
import { PreviewBlock } from "@/components/blocks/PreviewBlock";
import { WebBlock } from "@/components/blocks/WebBlock";

/** 初始化块系统 */
export function initBlockSystem(): void {
  // 注册终端块
  registerBlockType({
    type: "terminal",
    displayName: "终端",
    icon: "terminal",
    component: TerminalBlock,
    canCreate: true,
    defaultConfig: {
      type: "terminal",
      meta: { title: "终端" },
    },
  });

  // 注册预览块
  registerBlockType({
    type: "preview",
    displayName: "预览",
    icon: "file",
    component: PreviewBlock,
    canCreate: true,
    defaultConfig: {
      type: "preview",
      meta: { title: "预览" },
    },
  });

  // 注册 Web 块
  registerBlockType({
    type: "web",
    displayName: "Web",
    icon: "globe",
    component: WebBlock,
    canCreate: true,
    defaultConfig: {
      type: "web",
      meta: { title: "Web" },
    },
  });
}
