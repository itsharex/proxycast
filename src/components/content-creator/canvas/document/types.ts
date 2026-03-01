/**
 * @file 文档画布类型定义
 * @description 定义文档画布相关的核心类型
 * @module components/content-creator/canvas/document/types
 */

/**
 * 平台类型
 */
export type PlatformType = "wechat" | "xiaohongshu" | "zhihu" | "markdown";

/**
 * 导出格式
 */
export type ExportFormat = "markdown" | "text" | "clipboard";

/**
 * 文档版本
 */
export interface DocumentVersion {
  /** 版本 ID */
  id: string;
  /** 文档内容 */
  content: string;
  /** 创建时间戳 */
  createdAt: number;
  /** 版本描述 */
  description?: string;
}

/**
 * 文档画布状态
 */
export interface DocumentCanvasState {
  /** 画布类型标识 */
  type: "document";
  /** 当前文档内容 */
  content: string;
  /** 当前平台 */
  platform: PlatformType;
  /** 版本历史 */
  versions: DocumentVersion[];
  /** 当前版本 ID */
  currentVersionId: string;
  /** 是否处于编辑模式 */
  isEditing: boolean;
}

/**
 * 文档画布 Props
 */
export interface DocumentCanvasProps {
  /** 画布状态 */
  state: DocumentCanvasState;
  /** 状态变更回调 */
  onStateChange: (state: DocumentCanvasState) => void;
  /** 返回首页回调 */
  onBackHome?: () => void;
  /** 关闭画布回调 */
  onClose: () => void;
  /** 是否正在流式输出 */
  isStreaming?: boolean;
  /** 选中文本变更回调 */
  onSelectionTextChange?: (text: string) => void;
  /** 当前项目 ID（用于跨页面插图匹配） */
  projectId?: string | null;
  /** 当前文稿 ID（用于跨页面插图匹配） */
  contentId?: string | null;
  /** 自动配图的主题关键词 */
  autoImageTopic?: string;
}

/**
 * 文档工具栏 Props
 */
export interface DocumentToolbarProps {
  /** 当前版本 */
  currentVersion: DocumentVersion | null;
  /** 版本列表 */
  versions: DocumentVersion[];
  /** 是否处于编辑模式 */
  isEditing: boolean;
  /** 版本切换回调 */
  onVersionChange: (versionId: string) => void;
  /** 编辑模式切换回调 */
  onEditToggle: () => void;
  /** 保存回调 */
  onSave: () => void;
  /** 取消编辑回调 */
  onCancel: () => void;
  /** 导出回调 */
  onExport: (format: ExportFormat) => void;
  /** 主题自动配图 */
  onAutoInsertImages?: () => void;
  /** 自动配图执行中 */
  autoInsertLoading?: boolean;
  /** 关闭回调 */
  onClose: () => void;
}

/**
 * 文档渲染器 Props
 */
export interface DocumentRendererProps {
  /** 文档内容 */
  content: string;
  /** 平台类型 */
  platform: PlatformType;
  /** 是否正在流式输出 */
  isStreaming?: boolean;
  /** 选中文本变更回调 */
  onSelectionTextChange?: (text: string) => void;
}

/**
 * 平台标签 Props
 */
export interface PlatformTabsProps {
  /** 当前平台 */
  currentPlatform: PlatformType;
  /** 平台切换回调 */
  onPlatformChange: (platform: PlatformType) => void;
}

/**
 * 文档编辑器 Props
 */
export interface DocumentEditorProps {
  /** 文档内容 */
  content: string;
  /** 内容变更回调 */
  onChange: (content: string) => void;
  /** 保存回调 */
  onSave: () => void;
  /** 取消回调 */
  onCancel: () => void;
  /** 选中文本变更回调 */
  onSelectionTextChange?: (text: string) => void;
}

/**
 * 版本选择器 Props
 */
export interface VersionSelectorProps {
  /** 当前版本 */
  currentVersion: DocumentVersion | null;
  /** 版本列表 */
  versions: DocumentVersion[];
  /** 版本切换回调 */
  onVersionChange: (versionId: string) => void;
}

/**
 * 平台配置
 */
export interface PlatformConfig {
  id: PlatformType;
  name: string;
  icon: string;
  description: string;
}

/**
 * 平台配置列表
 */
export const PLATFORM_CONFIGS: PlatformConfig[] = [
  { id: "wechat", name: "公众号", icon: "📱", description: "微信公众号样式" },
  {
    id: "xiaohongshu",
    name: "小红书",
    icon: "📕",
    description: "小红书笔记样式",
  },
  { id: "zhihu", name: "知乎", icon: "📝", description: "知乎专栏样式" },
  {
    id: "markdown",
    name: "Markdown",
    icon: "📄",
    description: "原始 Markdown",
  },
];

/**
 * 创建初始文档画布状态
 */
export function createInitialDocumentState(
  content: string = "",
): DocumentCanvasState {
  const initialVersion: DocumentVersion = {
    id: crypto.randomUUID(),
    content,
    createdAt: Date.now(),
    description: "初始版本",
  };

  return {
    type: "document",
    content,
    platform: "markdown",
    versions: [initialVersion],
    currentVersionId: initialVersion.id,
    isEditing: false,
  };
}
