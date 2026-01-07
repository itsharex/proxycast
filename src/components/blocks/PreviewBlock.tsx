/**
 * @file PreviewBlock.tsx
 * @description 文件预览块组件
 * @module components/blocks/PreviewBlock
 *
 * 支持多种文件类型的预览。
 */

import React, { useState, useEffect } from "react";
import type { BlockComponentProps } from "@/lib/blocks/types";
import { BlockFrame } from "./BlockFrame";
import { MarkdownPreview } from "@/components/preview/MarkdownPreview";
import { ImagePreview } from "@/components/preview/ImagePreview";

/** 文件类型 */
type FileType = "markdown" | "image" | "text" | "code" | "unknown";

/** 根据文件扩展名获取文件类型 */
function getFileType(filePath: string): FileType {
  const ext = filePath.split(".").pop()?.toLowerCase() ?? "";

  const imageExts = ["png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "bmp"];
  const markdownExts = ["md", "markdown", "mdx"];
  const codeExts = [
    "js",
    "ts",
    "jsx",
    "tsx",
    "py",
    "rs",
    "go",
    "java",
    "c",
    "cpp",
    "h",
    "hpp",
    "css",
    "scss",
    "html",
    "json",
    "yaml",
    "yml",
    "toml",
    "xml",
  ];
  const textExts = ["txt", "log", "csv"];

  if (imageExts.includes(ext)) return "image";
  if (markdownExts.includes(ext)) return "markdown";
  if (codeExts.includes(ext)) return "code";
  if (textExts.includes(ext)) return "text";

  return "unknown";
}

/** 获取文件名 */
function getFileName(filePath: string): string {
  return filePath.split("/").pop() ?? filePath;
}

/**
 * 文件预览块组件
 */
export const PreviewBlock: React.FC<BlockComponentProps> = ({
  block,
  viewModel,
  visible = true,
}) => {
  const [content, setContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const filePath = block.config.filePath;
  const fileType = filePath ? getFileType(filePath) : "unknown";
  const fileName = filePath ? getFileName(filePath) : "未知文件";

  // 加载文件内容
  useEffect(() => {
    if (!filePath) {
      setError("未指定文件路径");
      return;
    }

    // 图片类型不需要加载内容
    if (fileType === "image") {
      return;
    }

    setLoading(true);
    setError(null);

    // TODO: 调用后端 API 读取文件内容
    // 这里暂时使用模拟数据
    setTimeout(() => {
      setContent(
        `# ${fileName}\n\n这是文件预览的占位内容。\n\n实际实现需要调用后端 API 读取文件。`,
      );
      setLoading(false);
    }, 500);
  }, [filePath, fileType, fileName]);

  // 渲染内容
  const renderContent = () => {
    if (loading) {
      return (
        <div className="preview-loading">
          <div className="preview-spinner" />
          <span>加载中...</span>
        </div>
      );
    }

    if (error) {
      return (
        <div className="preview-error">
          <svg
            className="w-12 h-12 mb-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          <span>{error}</span>
        </div>
      );
    }

    switch (fileType) {
      case "image":
        return <ImagePreview src={filePath!} alt={fileName} />;

      case "markdown":
        return <MarkdownPreview content={content ?? ""} />;

      case "code":
      case "text":
        return (
          <pre className="preview-code">
            <code>{content}</code>
          </pre>
        );

      default:
        return (
          <div className="preview-unsupported">
            <svg
              className="w-12 h-12 mb-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            <span>不支持预览此文件类型</span>
          </div>
        );
    }
  };

  return (
    <BlockFrame block={block} viewModel={viewModel} title={fileName}>
      <div className={`preview-block ${visible ? "" : "hidden"}`}>
        {renderContent()}
      </div>
    </BlockFrame>
  );
};

export default PreviewBlock;
