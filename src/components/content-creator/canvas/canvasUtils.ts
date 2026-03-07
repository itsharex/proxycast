/**
 * @file 画布工具函数
 * @description 画布相关的工具函数和类型定义
 * @module components/content-creator/canvas/canvasUtils
 */

import type { ThemeType } from "../types";
import { createInitialDocumentState } from "./document";
import type { DocumentCanvasState } from "./document/types";
import { createInitialPosterState } from "./poster";
import type { PosterCanvasState } from "./poster/types";
import { createInitialMusicState } from "./music/types";
import type { MusicCanvasState } from "./music/types";
import { createInitialScriptState } from "./script";
import type { ScriptCanvasState } from "./script/types";
import { createInitialNovelState } from "./novel";
import type { NovelCanvasState } from "./novel/types";
import { createInitialVideoState } from "./video";
import type { VideoCanvasState } from "./video/types";

/**
 * 画布状态联合类型
 */
export type CanvasStateUnion =
  | DocumentCanvasState
  | PosterCanvasState
  | MusicCanvasState
  | ScriptCanvasState
  | NovelCanvasState
  | VideoCanvasState;

/**
 * 画布类型
 */
export type CanvasType = "document" | "poster" | "music" | "script" | "novel" | "video";

/**
 * 主题到画布类型的映射
 * 所有主题都支持 document 画布，特定主题有专用画布
 *
 * 设计原则：
 * - 所有主题都可以触发画布（当检测到 <write_file> 标签时）
 * - 特定主题使用专用画布类型（如 music、poster、script）
 * - 通用主题（general、knowledge、planning）使用 document 画布
 */
const THEME_TO_CANVAS_TYPE: Record<ThemeType, CanvasType | null> = {
  general: "document", // 通用对话也支持文档画布
  "social-media": "document",
  poster: "poster",
  music: "music",
  knowledge: "document", // 知识探索支持文档画布
  planning: "document", // 计划规划支持文档画布
  document: "document",
  video: "video",
  novel: "novel",
};

/**
 * 获取主题对应的画布类型
 */
export function getCanvasTypeForTheme(theme: ThemeType): CanvasType | null {
  return THEME_TO_CANVAS_TYPE[theme];
}

/**
 * 判断主题是否支持画布
 */
export function isCanvasSupported(theme: ThemeType): boolean {
  return THEME_TO_CANVAS_TYPE[theme] !== null;
}

/**
 * 根据主题创建初始画布状态
 */
export function createInitialCanvasState(
  theme: ThemeType,
  content?: string,
): CanvasStateUnion | null {
  const canvasType = THEME_TO_CANVAS_TYPE[theme];

  switch (canvasType) {
    case "document":
      return createInitialDocumentState(content || "");
    case "poster":
      return createInitialPosterState();
    case "music":
      return createInitialMusicState();
    case "script":
      return createInitialScriptState(content);
    case "novel":
      return createInitialNovelState(content);
    case "video":
      return createInitialVideoState(content);
    default:
      return null;
  }
}
