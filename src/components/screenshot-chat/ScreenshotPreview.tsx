/**
 * @file ScreenshotPreview.tsx
 * @description 截图预览组件，用于悬浮对话窗口中显示截图
 * @module components/screenshot-chat/ScreenshotPreview
 */

import React, { useState, useCallback } from "react";

/** 截图预览属性 */
export interface ScreenshotPreviewProps {
  /** 图片路径或 Base64 编码 */
  src: string;
  /** 图片 alt 文本 */
  alt?: string;
  /** 自定义类名 */
  className?: string;
  /** 最大高度 */
  maxHeight?: number;
}

/**
 * 截图预览组件
 * 支持缩放和拖拽查看截图
 */
export const ScreenshotPreview: React.FC<ScreenshotPreviewProps> = ({
  src,
  alt = "截图预览",
  className = "",
  maxHeight = 300,
}) => {
  const [scale, setScale] = useState(1);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // 滚轮缩放
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.1 : 0.1;
    setScale((prev) => Math.max(0.5, Math.min(3, prev + delta)));
  }, []);

  // 拖拽开始
  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (scale > 1) {
        setIsDragging(true);
        setDragStart({ x: e.clientX - position.x, y: e.clientY - position.y });
      }
    },
    [scale, position],
  );

  // 拖拽中
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (isDragging) {
        setPosition({
          x: e.clientX - dragStart.x,
          y: e.clientY - dragStart.y,
        });
      }
    },
    [isDragging, dragStart],
  );

  // 拖拽结束
  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  // 重置缩放和位置
  const handleReset = useCallback(() => {
    setScale(1);
    setPosition({ x: 0, y: 0 });
  }, []);

  // 放大
  const handleZoomIn = useCallback(() => {
    setScale((prev) => Math.min(3, prev + 0.25));
  }, []);

  // 缩小
  const handleZoomOut = useCallback(() => {
    setScale((prev) => Math.max(0.5, prev - 0.25));
  }, []);

  return (
    <div className={`screenshot-preview ${className}`}>
      {/* 工具栏 */}
      <div className="screenshot-preview-toolbar">
        <button
          onClick={handleZoomOut}
          title="缩小"
          className="screenshot-preview-btn"
        >
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <line x1="8" y1="11" x2="14" y2="11" />
          </svg>
        </button>
        <span className="screenshot-preview-scale">
          {Math.round(scale * 100)}%
        </span>
        <button
          onClick={handleZoomIn}
          title="放大"
          className="screenshot-preview-btn"
        >
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
            <line x1="11" y1="8" x2="11" y2="14" />
            <line x1="8" y1="11" x2="14" y2="11" />
          </svg>
        </button>
        <button
          onClick={handleReset}
          title="重置"
          className="screenshot-preview-btn"
        >
          <svg
            className="w-4 h-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <polyline points="1 4 1 10 7 10" />
            <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10" />
          </svg>
        </button>
      </div>

      {/* 图片容器 */}
      <div
        className="screenshot-preview-container"
        style={{ maxHeight: `${maxHeight}px` }}
        onWheel={handleWheel}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <img
          src={src}
          alt={alt}
          className="screenshot-preview-image"
          style={{
            transform: `translate(${position.x}px, ${position.y}px) scale(${scale})`,
            transition: isDragging ? "none" : "transform 0.1s ease",
            cursor: scale > 1 ? (isDragging ? "grabbing" : "grab") : "default",
          }}
          draggable={false}
        />
      </div>
    </div>
  );
};

export default ScreenshotPreview;
