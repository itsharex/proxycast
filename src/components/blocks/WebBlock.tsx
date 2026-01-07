/**
 * @file WebBlock.tsx
 * @description Web 视图块组件
 * @module components/blocks/WebBlock
 *
 * 内嵌 Web 页面的块组件。
 */

import React, { useState, useCallback, useRef } from "react";
import type { BlockComponentProps } from "@/lib/blocks/types";
import { BlockFrame } from "./BlockFrame";

/**
 * Web 视图块组件
 */
export const WebBlock: React.FC<BlockComponentProps> = ({
  block,
  viewModel,
  visible = true,
}) => {
  const [url, setUrl] = useState(block.config.url ?? "");
  const [inputUrl, setInputUrl] = useState(block.config.url ?? "");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // 导航到 URL
  const handleNavigate = useCallback(() => {
    if (!inputUrl) return;

    let finalUrl = inputUrl;
    // 自动添加协议
    if (!finalUrl.startsWith("http://") && !finalUrl.startsWith("https://")) {
      finalUrl = "https://" + finalUrl;
    }

    setUrl(finalUrl);
    setLoading(true);
    setError(null);
  }, [inputUrl]);

  // 键盘事件
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        handleNavigate();
      }
    },
    [handleNavigate],
  );

  // iframe 加载完成
  const handleLoad = useCallback(() => {
    setLoading(false);
  }, []);

  // iframe 加载错误
  const handleError = useCallback(() => {
    setLoading(false);
    setError("无法加载页面");
  }, []);

  // 刷新
  const handleRefresh = useCallback(() => {
    if (iframeRef.current) {
      setLoading(true);
      iframeRef.current.src = url;
    }
  }, [url]);

  // 后退
  const handleBack = useCallback(() => {
    if (iframeRef.current?.contentWindow) {
      iframeRef.current.contentWindow.history.back();
    }
  }, []);

  // 前进
  const handleForward = useCallback(() => {
    if (iframeRef.current?.contentWindow) {
      iframeRef.current.contentWindow.history.forward();
    }
  }, []);

  // 自定义操作按钮
  const actions = (
    <>
      <button className="block-frame-btn" onClick={handleBack} title="后退">
        <svg
          className="w-3.5 h-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <polyline points="15 18 9 12 15 6" />
        </svg>
      </button>
      <button className="block-frame-btn" onClick={handleForward} title="前进">
        <svg
          className="w-3.5 h-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>
      <button className="block-frame-btn" onClick={handleRefresh} title="刷新">
        <svg
          className="w-3.5 h-3.5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
        </svg>
      </button>
    </>
  );

  return (
    <BlockFrame
      block={block}
      viewModel={viewModel}
      title="Web"
      actions={actions}
    >
      <div className={`web-block ${visible ? "" : "hidden"}`}>
        {/* 地址栏 */}
        <div className="web-block-addressbar">
          <input
            type="text"
            className="web-block-url-input"
            placeholder="输入 URL..."
            value={inputUrl}
            onChange={(e) => setInputUrl(e.target.value)}
            onKeyDown={handleKeyDown}
          />
          <button
            className="web-block-go-btn"
            onClick={handleNavigate}
            disabled={!inputUrl}
          >
            <svg
              className="w-4 h-4"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
            >
              <line x1="5" y1="12" x2="19" y2="12" />
              <polyline points="12 5 19 12 12 19" />
            </svg>
          </button>
        </div>

        {/* 内容区 */}
        <div className="web-block-content">
          {loading && (
            <div className="web-block-loading">
              <div className="preview-spinner" />
              <span>加载中...</span>
            </div>
          )}

          {error && (
            <div className="web-block-error">
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
          )}

          {!url && !loading && !error && (
            <div className="web-block-empty">
              <svg
                className="w-12 h-12 mb-4"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="2" y1="12" x2="22" y2="12" />
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
              </svg>
              <span>输入 URL 开始浏览</span>
            </div>
          )}

          {url && (
            <iframe
              ref={iframeRef}
              src={url}
              className="web-block-iframe"
              onLoad={handleLoad}
              onError={handleError}
              sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
            />
          )}
        </div>
      </div>
    </BlockFrame>
  );
};

export default WebBlock;
