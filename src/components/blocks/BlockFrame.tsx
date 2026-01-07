/**
 * @file BlockFrame.tsx
 * @description 块框架组件
 * @module components/blocks/BlockFrame
 *
 * 提供块的通用框架，包括标题栏、操作按钮等。
 */

import React, { useCallback } from "react";
import type { Block, BlockViewModel } from "@/lib/blocks/types";
import { getBlockDisplayName, getBlockIcon } from "@/lib/blocks/blockStore";

/** 块框架属性 */
export interface BlockFrameProps {
  /** 块实例 */
  block: Block;
  /** 视图模型 */
  viewModel: BlockViewModel;
  /** 子组件 */
  children: React.ReactNode;
  /** 是否显示标题栏 */
  showHeader?: boolean;
  /** 自定义标题 */
  title?: string;
  /** 自定义操作按钮 */
  actions?: React.ReactNode;
}

/** 图标组件 */
const BlockIcon: React.FC<{ type: string; className?: string }> = ({
  type,
  className,
}) => {
  const iconMap: Record<string, React.ReactNode> = {
    terminal: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </svg>
    ),
    file: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <polyline points="14 2 14 8 20 8" />
      </svg>
    ),
    globe: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <circle cx="12" cy="12" r="10" />
        <line x1="2" y1="12" x2="22" y2="12" />
        <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
      </svg>
    ),
    sparkles: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <path d="M12 3l1.5 4.5L18 9l-4.5 1.5L12 15l-1.5-4.5L6 9l4.5-1.5L12 3z" />
        <path d="M5 19l1 3 1-3 3-1-3-1-1-3-1 3-3 1 3 1z" />
        <path d="M19 12l1 2 1-2 2-1-2-1-1-2-1 2-2 1 2 1z" />
      </svg>
    ),
    "file-text": (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
        <polyline points="14 2 14 8 20 8" />
        <line x1="16" y1="13" x2="8" y2="13" />
        <line x1="16" y1="17" x2="8" y2="17" />
        <polyline points="10 9 9 9 8 9" />
      </svg>
    ),
    image: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <circle cx="8.5" cy="8.5" r="1.5" />
        <polyline points="21 15 16 10 5 21" />
      </svg>
    ),
    square: (
      <svg
        className={className}
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
      </svg>
    ),
  };

  return <>{iconMap[type] ?? iconMap.square}</>;
};

/** 关闭按钮 */
const CloseButton: React.FC<{ onClick: () => void }> = ({ onClick }) => (
  <button
    className="block-frame-btn block-frame-close"
    onClick={onClick}
    title="关闭"
  >
    <svg
      className="w-3.5 h-3.5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <line x1="18" y1="6" x2="6" y2="18" />
      <line x1="6" y1="6" x2="18" y2="18" />
    </svg>
  </button>
);

/** 最大化按钮 */
const MaximizeButton: React.FC<{
  isMagnified: boolean;
  onClick: () => void;
}> = ({ isMagnified, onClick }) => (
  <button
    className="block-frame-btn"
    onClick={onClick}
    title={isMagnified ? "还原" : "最大化"}
  >
    {isMagnified ? (
      <svg
        className="w-3.5 h-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <polyline points="4 14 10 14 10 20" />
        <polyline points="20 10 14 10 14 4" />
        <line x1="14" y1="10" x2="21" y2="3" />
        <line x1="3" y1="21" x2="10" y2="14" />
      </svg>
    ) : (
      <svg
        className="w-3.5 h-3.5"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <polyline points="15 3 21 3 21 9" />
        <polyline points="9 21 3 21 3 15" />
        <line x1="21" y1="3" x2="14" y2="10" />
        <line x1="3" y1="21" x2="10" y2="14" />
      </svg>
    )}
  </button>
);

/**
 * 块框架组件
 */
export const BlockFrame: React.FC<BlockFrameProps> = ({
  block,
  viewModel,
  children,
  showHeader = true,
  title,
  actions,
}) => {
  const handleClose = useCallback(() => {
    viewModel.close();
  }, [viewModel]);

  const handleMaximize = useCallback(() => {
    viewModel.toggleMagnify();
  }, [viewModel]);

  const handleFocus = useCallback(() => {
    viewModel.focus();
  }, [viewModel]);

  const displayTitle =
    title ?? block.meta.title ?? getBlockDisplayName(block.type);
  const iconType = block.meta.icon ?? getBlockIcon(block.type);

  return (
    <div
      className={`block-frame ${viewModel.isFocused ? "focused" : ""} ${viewModel.isMagnified ? "magnified" : ""}`}
      onClick={handleFocus}
    >
      {showHeader && (
        <div className="block-frame-header">
          <div className="block-frame-title">
            <BlockIcon type={iconType} className="block-frame-icon" />
            <span className="block-frame-title-text">{displayTitle}</span>
          </div>
          <div className="block-frame-actions">
            {actions}
            <MaximizeButton
              isMagnified={viewModel.isMagnified}
              onClick={handleMaximize}
            />
            <CloseButton onClick={handleClose} />
          </div>
        </div>
      )}
      <div className="block-frame-content">{children}</div>
    </div>
  );
};

export default BlockFrame;
