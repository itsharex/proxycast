/**
 * @file TerminalBlock.tsx
 * @description 终端块组件
 * @module components/blocks/TerminalBlock
 *
 * 将终端封装为块系统的一部分。
 */

import React, { useEffect, useRef, useCallback } from "react";
import { useAtomValue } from "jotai";
import type { BlockComponentProps } from "@/lib/blocks/types";
import { themeAtom } from "@/lib/blocks/blockStore";
import { TermWrap } from "@/components/terminal/termwrap";
import { type ThemeName } from "@/lib/terminal/themes";
import { BlockFrame } from "./BlockFrame";
import "@xterm/xterm/css/xterm.css";

/**
 * 终端块组件
 */
export const TerminalBlock: React.FC<BlockComponentProps> = ({
  block,
  viewModel,
  visible = true,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const termWrapRef = useRef<TermWrap | null>(null);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const currentTheme = useAtomValue(themeAtom) as ThemeName;

  // 获取会话 ID
  const sessionId = block.config.sessionId;

  useEffect(() => {
    const container = containerRef.current;
    if (!container || !sessionId) return;

    // 创建 TermWrap 实例
    const termWrap = new TermWrap(sessionId, container, {
      themeName: currentTheme,
    });

    termWrapRef.current = termWrap;

    // 设置 ResizeObserver
    const rszObs = new ResizeObserver(() => {
      termWrap.handleResize_debounced();
    });
    rszObs.observe(container);
    resizeObserverRef.current = rszObs;

    return () => {
      termWrap.dispose();
      rszObs.disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  // 主题变化时更新
  useEffect(() => {
    if (termWrapRef.current && currentTheme) {
      termWrapRef.current.setTheme(currentTheme);
    }
  }, [currentTheme]);

  // 可见性变化时处理
  useEffect(() => {
    if (visible && termWrapRef.current) {
      termWrapRef.current.handleResize_debounced();
      if (viewModel.isFocused) {
        termWrapRef.current.focus();
      }
    }
  }, [visible, viewModel.isFocused]);

  // 聚焦时聚焦终端
  useEffect(() => {
    if (viewModel.isFocused && termWrapRef.current) {
      termWrapRef.current.focus();
    }
  }, [viewModel.isFocused]);

  const handleContainerClick = useCallback(() => {
    termWrapRef.current?.focus();
  }, []);

  if (!sessionId) {
    return (
      <BlockFrame block={block} viewModel={viewModel}>
        <div className="flex items-center justify-center h-full text-gray-500">
          无效的终端会话
        </div>
      </BlockFrame>
    );
  }

  return (
    <BlockFrame
      block={block}
      viewModel={viewModel}
      title={block.meta.title ?? "终端"}
    >
      <div
        ref={containerRef}
        className={`terminal-block-container ${visible ? "" : "hidden"}`}
        onClick={handleContainerClick}
        style={{ height: "100%", width: "100%" }}
      />
    </BlockFrame>
  );
};

export default TerminalBlock;
