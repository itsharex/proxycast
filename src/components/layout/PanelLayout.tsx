/**
 * @file PanelLayout.tsx
 * @description 分屏布局组件
 * @module components/layout/PanelLayout
 *
 * 使用 react-resizable-panels 实现可调整的分屏布局。
 */

import React from "react";
import {
  Panel,
  Group as ResizablePanelGroup,
  Separator as ResizableHandle,
} from "react-resizable-panels";
import type { PanelNode, Block, BlockViewModel } from "@/lib/blocks/types";
import { blockRegistry } from "@/lib/blocks/registry";

/** 布局属性 */
export interface PanelLayoutProps {
  /** 根节点 */
  rootNode: PanelNode;
  /** 块映射 */
  blocks: Map<string, Block>;
  /** 活动块 ID */
  activeBlockId?: string;
  /** 最大化块 ID */
  magnifiedBlockId?: string;
  /** 块聚焦回调 */
  onBlockFocus?: (blockId: string) => void;
  /** 块关闭回调 */
  onBlockClose?: (blockId: string) => void;
  /** 块最大化回调 */
  onBlockMagnify?: (blockId: string) => void;
  /** 面板大小变化回调 */
  onPanelResize?: (nodeId: string, size: number) => void;
}

/** 调整手柄组件 */
const ResizeHandle: React.FC<{ orientation: "horizontal" | "vertical" }> = ({
  orientation,
}) => (
  <ResizableHandle
    className={`panel-resize-handle ${orientation === "horizontal" ? "horizontal" : "vertical"}`}
  >
    <div className="panel-resize-handle-inner" />
  </ResizableHandle>
);

/** 块渲染器 */
const BlockRenderer: React.FC<{
  block: Block;
  viewModel: BlockViewModel;
  visible: boolean;
}> = ({ block, viewModel, visible }) => {
  const Component = blockRegistry.getComponent(block.type);

  if (!Component) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500">
        未知的块类型: {block.type}
      </div>
    );
  }

  return <Component block={block} viewModel={viewModel} visible={visible} />;
};

/** 面板节点渲染器 */
const PanelNodeRenderer: React.FC<{
  node: PanelNode;
  blocks: Map<string, Block>;
  activeBlockId?: string;
  magnifiedBlockId?: string;
  onBlockFocus?: (blockId: string) => void;
  onBlockClose?: (blockId: string) => void;
  onBlockMagnify?: (blockId: string) => void;
  onPanelResize?: (nodeId: string, size: number) => void;
}> = ({
  node,
  blocks,
  activeBlockId,
  magnifiedBlockId,
  onBlockFocus,
  onBlockClose,
  onBlockMagnify,
  onPanelResize,
}) => {
  // 块节点
  if (node.type === "block" && node.blockId) {
    const block = blocks.get(node.blockId);
    if (!block) {
      return (
        <div className="flex items-center justify-center h-full text-gray-500">
          块不存在
        </div>
      );
    }

    const viewModel: BlockViewModel = {
      blockId: block.id,
      isFocused: activeBlockId === block.id,
      isMagnified: magnifiedBlockId === block.id,
      focus: () => onBlockFocus?.(block.id),
      toggleMagnify: () => onBlockMagnify?.(block.id),
      close: () => onBlockClose?.(block.id),
    };

    return (
      <BlockRenderer
        block={block}
        viewModel={viewModel}
        visible={!magnifiedBlockId || magnifiedBlockId === block.id}
      />
    );
  }

  // 组节点
  if (node.type === "group" && node.children && node.children.length > 0) {
    const orientation = node.direction ?? "horizontal";

    return (
      <ResizablePanelGroup orientation={orientation} className="panel-group">
        {node.children.map((child, index) => (
          <React.Fragment key={child.id}>
            {index > 0 && <ResizeHandle orientation={orientation} />}
            <Panel
              defaultSize={child.size ?? 100 / node.children!.length}
              minSize={10}
              onResize={(size) => onPanelResize?.(child.id, size.asPercentage)}
            >
              <PanelNodeRenderer
                node={child}
                blocks={blocks}
                activeBlockId={activeBlockId}
                magnifiedBlockId={magnifiedBlockId}
                onBlockFocus={onBlockFocus}
                onBlockClose={onBlockClose}
                onBlockMagnify={onBlockMagnify}
                onPanelResize={onPanelResize}
              />
            </Panel>
          </React.Fragment>
        ))}
      </ResizablePanelGroup>
    );
  }

  // 空节点
  return (
    <div className="flex items-center justify-center h-full text-gray-500">
      空面板
    </div>
  );
};

/**
 * 分屏布局组件
 */
export const PanelLayout: React.FC<PanelLayoutProps> = ({
  rootNode,
  blocks,
  activeBlockId,
  magnifiedBlockId,
  onBlockFocus,
  onBlockClose,
  onBlockMagnify,
  onPanelResize,
}) => {
  // 如果有最大化的块，只显示该块
  if (magnifiedBlockId) {
    const block = blocks.get(magnifiedBlockId);
    if (block) {
      const viewModel: BlockViewModel = {
        blockId: block.id,
        isFocused: true,
        isMagnified: true,
        focus: () => onBlockFocus?.(block.id),
        toggleMagnify: () => onBlockMagnify?.(block.id),
        close: () => onBlockClose?.(block.id),
      };

      return (
        <div className="panel-layout magnified">
          <BlockRenderer block={block} viewModel={viewModel} visible={true} />
        </div>
      );
    }
  }

  return (
    <div className="panel-layout">
      <PanelNodeRenderer
        node={rootNode}
        blocks={blocks}
        activeBlockId={activeBlockId}
        magnifiedBlockId={magnifiedBlockId}
        onBlockFocus={onBlockFocus}
        onBlockClose={onBlockClose}
        onBlockMagnify={onBlockMagnify}
        onPanelResize={onPanelResize}
      />
    </div>
  );
};

export default PanelLayout;
