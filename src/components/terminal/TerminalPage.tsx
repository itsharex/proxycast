/**
 * @file TerminalPage.tsx
 * @description 内置终端页面组件 - 后端预创建架构
 * @module components/terminal/TerminalPage
 *
 * ## 架构说明
 * PTY 在后端预创建，前端只负责连接。
 * 新建终端时先调用后端创建会话，成功后再创建 UI 组件。
 *
 * ## 功能特性
 * - 多标签页管理
 * - 终端搜索 (Ctrl+F)
 * - 主题切换
 */

import React, { useEffect, useRef, useState, useCallback } from "react";
import "@xterm/xterm/css/xterm.css";
import {
  createTerminalSession,
  closeTerminal,
  type SessionStatus,
} from "@/lib/terminal-api";
import { TermWrap } from "./termwrap";
import { TerminalSearch } from "./TerminalSearch";
import {
  type ThemeName,
  getThemeList,
  saveThemePreference,
  loadThemePreference,
} from "@/lib/terminal/themes";
import "./terminal.css";

// ============================================================================
// 类型定义
// ============================================================================

/** 标签页数据 */
interface Tab {
  id: string;
  sessionId: string;
  title: string;
  status: SessionStatus;
  isSSH?: boolean;
}

// ============================================================================
// 子组件
// ============================================================================

/** 状态指示器 */
const StatusIndicator: React.FC<{ status: SessionStatus }> = ({ status }) => {
  const statusClasses: Record<SessionStatus, string> = {
    connecting: "connecting",
    running: "running",
    done: "done",
    error: "error",
  };
  return (
    <span
      className={`terminal-status-indicator ${statusClasses[status] || ""}`}
      title={status}
    />
  );
};

/** 终端图标 */
const TerminalIcon: React.FC<{ isSSH?: boolean; className?: string }> = ({
  isSSH,
  className,
}) => (
  <svg
    className={className || "w-4 h-4 mr-2 flex-shrink-0"}
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    {isSSH ? (
      <>
        <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
        <path d="M7 11V7a5 5 0 0 1 10 0v4" />
      </>
    ) : (
      <>
        <polyline points="4 17 10 11 4 5" />
        <line x1="12" y1="19" x2="20" y2="19" />
      </>
    )}
  </svg>
);

/** 关闭按钮 */
const CloseButton: React.FC<{ onClick: (e: React.MouseEvent) => void }> = ({
  onClick,
}) => (
  <button className="terminal-close-btn" onClick={onClick} title="关闭标签页">
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

/** 新建标签按钮 */
const NewTabButton: React.FC<{ onClick: () => void; disabled?: boolean }> = ({
  onClick,
  disabled,
}) => (
  <button
    className="terminal-new-tab-btn"
    onClick={onClick}
    disabled={disabled}
    title="新建终端"
  >
    <svg
      className="w-4 h-4"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
    >
      <line x1="12" y1="5" x2="12" y2="19" />
      <line x1="5" y1="12" x2="19" y2="12" />
    </svg>
  </button>
);

/** 搜索按钮 */
const SearchButton: React.FC<{ onClick: () => void }> = ({ onClick }) => (
  <button
    className="terminal-new-tab-btn"
    onClick={onClick}
    title="搜索 (Ctrl+F)"
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
    </svg>
  </button>
);

/** 主题选择器 */
const ThemeSelector: React.FC<{
  currentTheme: ThemeName;
  onThemeChange: (theme: ThemeName) => void;
}> = ({ currentTheme, onThemeChange }) => {
  const [isOpen, setIsOpen] = useState(false);
  const themes = getThemeList();

  return (
    <div className="relative">
      <button
        className="terminal-new-tab-btn"
        onClick={() => setIsOpen(!isOpen)}
        title="切换主题"
      >
        <svg
          className="w-4 h-4"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
        >
          <circle cx="12" cy="12" r="5" />
          <line x1="12" y1="1" x2="12" y2="3" />
          <line x1="12" y1="21" x2="12" y2="23" />
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
          <line x1="1" y1="12" x2="3" y2="12" />
          <line x1="21" y1="12" x2="23" y2="12" />
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
        </svg>
      </button>
      {isOpen && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
          />
          <div className="terminal-theme-dropdown">
            {themes.map((theme) => (
              <div
                key={theme.name}
                className={`terminal-theme-item ${theme.name === currentTheme ? "active" : ""}`}
                onClick={() => {
                  onThemeChange(theme.name);
                  setIsOpen(false);
                }}
              >
                <div
                  className="terminal-theme-preview"
                  style={{ backgroundColor: theme.theme.background }}
                />
                {theme.displayName}
              </div>
            ))}
          </div>
        </>
      )}
    </div>
  );
};

/** 标签页组件 */
const TabItem: React.FC<{
  tab: Tab;
  isActive: boolean;
  onSelect: () => void;
  onClose: () => void;
}> = ({ tab, isActive, onSelect, onClose }) => {
  const handleClose = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onClose();
    },
    [onClose],
  );

  return (
    <div
      className={`terminal-tab ${isActive ? "active" : ""}`}
      onClick={onSelect}
      role="tab"
      aria-selected={isActive}
    >
      <StatusIndicator status={tab.status} />
      <TerminalIcon isSSH={tab.isSSH} />
      <span className="truncate flex-1 text-left">{tab.title}</span>
      <CloseButton onClick={handleClose} />
    </div>
  );
};

/** 标签栏组件 */
const TerminalTabs: React.FC<{
  tabs: Tab[];
  activeTabId: string | null;
  onTabSelect: (id: string) => void;
  onTabClose: (id: string) => void;
  onNewTab: () => void;
  onSearchClick: () => void;
  currentTheme: ThemeName;
  onThemeChange: (theme: ThemeName) => void;
  isCreating?: boolean;
}> = ({
  tabs,
  activeTabId,
  onTabSelect,
  onTabClose,
  onNewTab,
  onSearchClick,
  currentTheme,
  onThemeChange,
  isCreating,
}) => (
  <div className="terminal-tabbar" role="tablist">
    <div className="flex items-center flex-1 overflow-x-auto h-full">
      {tabs.map((tab) => (
        <TabItem
          key={tab.id}
          tab={tab}
          isActive={tab.id === activeTabId}
          onSelect={() => onTabSelect(tab.id)}
          onClose={() => onTabClose(tab.id)}
        />
      ))}
    </div>
    <SearchButton onClick={onSearchClick} />
    <ThemeSelector currentTheme={currentTheme} onThemeChange={onThemeChange} />
    <NewTabButton onClick={onNewTab} disabled={isCreating} />
  </div>
);

/** 空状态占位符 */
const EmptyTabsPlaceholder: React.FC<{
  onNewTab: () => void;
  isCreating?: boolean;
}> = ({ onNewTab, isCreating }) => (
  <div className="terminal-empty-state">
    <TerminalIcon className="terminal-empty-state-icon" />
    <p className="terminal-empty-state-text">没有打开的终端</p>
    <button
      className="terminal-empty-state-btn"
      onClick={onNewTab}
      disabled={isCreating}
    >
      {isCreating ? "创建中..." : "新建终端"}
    </button>
  </div>
);

// ============================================================================
// 终端视图组件 - 连接模式
// ============================================================================

interface TerminalViewProps {
  /** 会话 ID（必须） */
  sessionId: string;
  /** 状态变化回调 */
  onStatusChange: (status: SessionStatus) => void;
  /** 是否自动聚焦 */
  autoFocus?: boolean;
  /** 是否可见（用于多标签页切换） */
  visible?: boolean;
  /** 主题名称 */
  themeName?: ThemeName;
  /** TermWrap 引用回调 */
  onTermWrapRef?: (termWrap: TermWrap | null) => void;
}

const TerminalView: React.FC<TerminalViewProps> = ({
  sessionId,
  onStatusChange,
  autoFocus,
  visible = true,
  themeName,
  onTermWrapRef,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const termWrapRef = useRef<TermWrap | null>(null);
  const resizeObserverRef = useRef<ResizeObserver | null>(null);
  const callbacksRef = useRef({ onStatusChange, onTermWrapRef });
  callbacksRef.current = { onStatusChange, onTermWrapRef };

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // 创建 TermWrap 实例
    const termWrap = new TermWrap(sessionId, container, {
      onStatusChange: (status) => callbacksRef.current.onStatusChange(status),
      themeName,
    });

    termWrapRef.current = termWrap;
    callbacksRef.current.onTermWrapRef?.(termWrap);

    // 设置 ResizeObserver
    const rszObs = new ResizeObserver(() => {
      termWrap.handleResize_debounced();
    });
    rszObs.observe(container);
    resizeObserverRef.current = rszObs;

    // 自动聚焦
    if (autoFocus) {
      setTimeout(() => termWrap.focus(), 10);
    }

    return () => {
      termWrap.dispose();
      rszObs.disconnect();
      callbacksRef.current.onTermWrapRef?.(null);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);

  // 主题变化时更新
  useEffect(() => {
    if (themeName && termWrapRef.current) {
      termWrapRef.current.setTheme(themeName);
    }
  }, [themeName]);

  // 当可见性变化时，触发 resize 和聚焦
  useEffect(() => {
    if (visible && termWrapRef.current) {
      termWrapRef.current.handleResize_debounced();
      if (autoFocus) {
        termWrapRef.current.focus();
      }
    }
  }, [visible, autoFocus, themeName]);

  return (
    <div
      ref={containerRef}
      className={`terminal-container ${visible ? "" : "terminal-hidden"}`}
      onClick={() => termWrapRef.current?.focus()}
    />
  );
};

// ============================================================================
// 主组件
// ============================================================================

export function TerminalPage() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [activeTabId, setActiveTabId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [currentTheme, setCurrentTheme] = useState<ThemeName>(
    loadThemePreference(),
  );
  const tabIdCounter = useRef(0);
  const termWrapRefs = useRef<Map<string, TermWrap>>(new Map());
  const pageRef = useRef<HTMLDivElement>(null);

  // 调试：打印布局链高度
  useEffect(() => {
    if (pageRef.current) {
      let el: HTMLElement | null = pageRef.current;
      const heights: string[] = [];
      while (el) {
        const style = window.getComputedStyle(el);
        heights.push(
          `${el.className?.slice(0, 30) || el.tagName}: ${style.height}`,
        );
        el = el.parentElement;
      }
      console.log("[TerminalPage] 布局链高度:", heights);
    }
  }, [tabs.length]);

  // 获取当前活动的 TermWrap
  const getActiveTermWrap = useCallback(() => {
    if (!activeTabId) return null;
    const tab = tabs.find((t) => t.id === activeTabId);
    if (!tab) return null;
    return termWrapRefs.current.get(tab.sessionId) ?? null;
  }, [activeTabId, tabs]);

  // 处理 TermWrap 引用
  const handleTermWrapRef = useCallback(
    (sessionId: string, termWrap: TermWrap | null) => {
      if (termWrap) {
        termWrapRefs.current.set(sessionId, termWrap);
      } else {
        termWrapRefs.current.delete(sessionId);
      }
    },
    [],
  );

  // 创建新终端
  const handleNewTerminal = useCallback(async () => {
    if (isCreating) return;

    setIsCreating(true);
    setError(null);

    try {
      // 先调用后端创建会话
      const sessionId = await createTerminalSession();
      console.log("[TerminalPage] 会话已创建:", sessionId);

      // 创建成功后添加标签页
      const newTabId = `tab-${++tabIdCounter.current}`;
      const newTab: Tab = {
        id: newTabId,
        sessionId,
        title: "Terminal",
        status: "running",
        isSSH: false,
      };

      setTabs((prev) => [...prev, newTab]);
      setActiveTabId(newTabId);
    } catch (err) {
      console.error("[TerminalPage] 创建终端失败:", err);
      setError("创建终端会话失败");
    } finally {
      setIsCreating(false);
    }
  }, [isCreating]);

  // 状态变化
  const handleStatusChange = useCallback(
    (tabId: string, status: SessionStatus) => {
      setTabs((prev) =>
        prev.map((tab) => (tab.id === tabId ? { ...tab, status } : tab)),
      );
    },
    [],
  );

  // 关闭会话
  const handleCloseTab = useCallback(
    async (tabId: string) => {
      const tab = tabs.find((t) => t.id === tabId);
      if (tab) {
        try {
          await closeTerminal(tab.sessionId);
        } catch (err) {
          console.error("[TerminalPage] 关闭终端会话失败:", err);
        }
      }

      setTabs((prev) => {
        const remaining = prev.filter((t) => t.id !== tabId);
        // 如果关闭的是当前活动标签，切换到最后一个
        if (activeTabId === tabId && remaining.length > 0) {
          setActiveTabId(remaining[remaining.length - 1].id);
        } else if (remaining.length === 0) {
          setActiveTabId(null);
        }
        return remaining;
      });
    },
    [tabs, activeTabId],
  );

  // 主题变化
  const handleThemeChange = useCallback((theme: ThemeName) => {
    setCurrentTheme(theme);
    saveThemePreference(theme);
    // 更新所有终端的主题
    termWrapRefs.current.forEach((termWrap) => {
      termWrap.setTheme(theme);
    });
  }, []);

  // 搜索功能
  const handleSearch = useCallback(
    (term: string, options: import("@xterm/addon-search").ISearchOptions) => {
      const termWrap = getActiveTermWrap();
      if (!termWrap) return false;
      return termWrap.search(term, options);
    },
    [getActiveTermWrap],
  );

  const handleSearchNext = useCallback(
    (term: string, options: import("@xterm/addon-search").ISearchOptions) => {
      const termWrap = getActiveTermWrap();
      if (!termWrap) return false;
      return termWrap.searchNext(term, options);
    },
    [getActiveTermWrap],
  );

  const handleSearchPrevious = useCallback(
    (term: string, options: import("@xterm/addon-search").ISearchOptions) => {
      const termWrap = getActiveTermWrap();
      if (!termWrap) return false;
      return termWrap.searchPrevious(term, options);
    },
    [getActiveTermWrap],
  );

  const handleClearSearch = useCallback(() => {
    const termWrap = getActiveTermWrap();
    if (termWrap) {
      termWrap.clearSearch();
    }
  }, [getActiveTermWrap]);

  // 键盘快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+F 打开搜索
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        setShowSearch(true);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // 清除错误
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  // 没有标签页时显示空状态
  if (tabs.length === 0) {
    return (
      <div className="flex flex-col h-full w-full min-h-0 terminal-bg">
        <TerminalTabs
          tabs={[]}
          activeTabId={null}
          onTabSelect={() => {}}
          onTabClose={() => {}}
          onNewTab={handleNewTerminal}
          onSearchClick={() => setShowSearch(true)}
          currentTheme={currentTheme}
          onThemeChange={handleThemeChange}
          isCreating={isCreating}
        />
        <div className="flex-1">
          <EmptyTabsPlaceholder
            onNewTab={handleNewTerminal}
            isCreating={isCreating}
          />
        </div>
        {error && (
          <div className="absolute bottom-4 right-4 bg-red-600/90 text-white px-4 py-2 rounded-lg shadow-lg text-sm z-50">
            {error}
          </div>
        )}
      </div>
    );
  }

  return (
    <div
      ref={pageRef}
      className="flex flex-col w-full h-full overflow-hidden relative terminal-bg"
    >
      {/* 标签栏 */}
      <TerminalTabs
        tabs={tabs}
        activeTabId={activeTabId}
        onTabSelect={setActiveTabId}
        onTabClose={handleCloseTab}
        onNewTab={handleNewTerminal}
        onSearchClick={() => setShowSearch(true)}
        currentTheme={currentTheme}
        onThemeChange={handleThemeChange}
        isCreating={isCreating}
      />

      {/* 搜索栏 */}
      <TerminalSearch
        visible={showSearch}
        onClose={() => setShowSearch(false)}
        onSearch={handleSearch}
        onSearchNext={handleSearchNext}
        onSearchPrevious={handleSearchPrevious}
        onClearSearch={handleClearSearch}
      />

      {/* 终端视图 - 只渲染当前活动的终端（参考 waveterm） */}
      <div className="flex-1 min-h-0 relative flex flex-col">
        {tabs
          .filter((tab) => tab.id === activeTabId)
          .map((tab) => (
            <TerminalView
              key={tab.id}
              sessionId={tab.sessionId}
              onStatusChange={(status) => handleStatusChange(tab.id, status)}
              visible={true}
              autoFocus={true}
              themeName={currentTheme}
              onTermWrapRef={(termWrap) =>
                handleTermWrapRef(tab.sessionId, termWrap)
              }
            />
          ))}
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="absolute bottom-4 right-4 bg-red-600/90 text-white px-4 py-2 rounded-lg shadow-lg text-sm z-50">
          {error}
        </div>
      )}
    </div>
  );
}

export default TerminalPage;
