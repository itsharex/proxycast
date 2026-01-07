/**
 * @file termwrap.ts
 * @description 终端封装类 - 连接模式
 * @module components/terminal/termwrap
 *
 * 封装 xterm.js 终端实例，管理终端生命周期、大小同步、输入输出。
 *
 * ## 架构说明
 * PTY 在后端预创建，TermWrap 只负责"连接"到已存在的会话。
 * 构造函数接收 sessionId（必须），立即设置事件监听和输入处理。
 * resize 只负责同步大小到后端，不触发创建。
 *
 * ## 功能特性
 * - 搜索功能：集成 @xterm/addon-search
 * - 主题系统：支持多种终端主题切换
 * - Web 链接：自动识别并可点击 URL
 */

import { Terminal } from "@xterm/xterm";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { SearchAddon, type ISearchOptions } from "@xterm/addon-search";
import { FitAddon } from "./fitaddon";
import {
  resizeTerminal,
  writeToTerminalRaw,
  onSessionOutput,
  onSessionStatus,
  decodeBytes,
  encodeBase64,
  type SessionStatus,
} from "@/lib/terminal-api";
import {
  type ThemeName,
  getTheme,
  loadThemePreference,
} from "@/lib/terminal/themes";

/** 简单的 debounce 实现 */
function debounce<T extends (...args: unknown[]) => void>(
  fn: T,
  delay: number,
): T {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  return ((...args: unknown[]) => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  }) as T;
}

/** 终端配置选项 */
export interface TermWrapOptions {
  /** 终端字体大小 */
  fontSize?: number;
  /** 终端字体 */
  fontFamily?: string;
  /** 主题名称 */
  themeName?: ThemeName;
  /** 状态变化回调 */
  onStatusChange?: (status: SessionStatus) => void;
}

/** 搜索结果回调 */
export interface SearchCallbacks {
  /** 搜索结果变化 */
  onSearchResults?: (results: {
    resultIndex: number;
    resultCount: number;
  }) => void;
}

/**
 * 终端封装类 - 连接模式
 *
 * PTY 在后端预创建，TermWrap 只负责"连接"到已存在的会话。
 *
 * 使用方式：
 * 1. 先调用 createTerminalSession() 获取 sessionId
 * 2. 创建 TermWrap 实例，传入 sessionId
 * 3. TermWrap 自动连接并设置事件监听
 * 4. 首次 fit 后自动同步实际大小到后端
 */
export class TermWrap {
  /** 会话 ID（必须） */
  readonly sessionId: string;
  /** xterm 终端实例 */
  terminal: Terminal;
  /** 连接的 DOM 元素 */
  connectElem: HTMLDivElement;
  /** FitAddon 实例 */
  fitAddon: FitAddon;
  /** SearchAddon 实例 */
  searchAddon: SearchAddon;
  /** 是否已连接（事件监听已设置） */
  connected: boolean = false;
  /** 防抖的 resize 处理函数 */
  handleResize_debounced: () => void;
  /** 配置选项 */
  private options: TermWrapOptions;
  /** 当前主题名称 */
  private currentTheme: ThemeName;
  /** 搜索回调 */
  private searchCallbacks: SearchCallbacks = {};
  /** 需要清理的资源 */
  private toDispose: Array<{ dispose: () => void }> = [];
  /** 事件监听器清理函数 */
  private unlistenOutput?: () => void;
  private unlistenStatus?: () => void;
  /** 上次同步到后端的大小 */
  private lastSyncedSize: { rows: number; cols: number } | null = null;

  /**
   * 创建终端封装实例
   *
   * @param sessionId - 已创建的会话 ID（必须）
   * @param connectElem - 要挂载终端的 DOM 元素
   * @param options - 配置选项
   */
  constructor(
    sessionId: string,
    connectElem: HTMLDivElement,
    options: TermWrapOptions = {},
  ) {
    this.sessionId = sessionId;
    this.connectElem = connectElem;
    this.options = options;

    // 加载主题
    this.currentTheme = options.themeName ?? loadThemePreference();
    const theme = getTheme(this.currentTheme);

    // 创建终端实例
    this.terminal = new Terminal({
      cursorBlink: true,
      fontSize: options.fontSize ?? 14,
      fontFamily:
        options.fontFamily ?? 'Hack, Menlo, Monaco, "Courier New", monospace',
      theme,
      allowProposedApi: true,
      allowTransparency: true,
      scrollback: 5000,
      drawBoldTextInBrightColors: false,
      fontWeight: "normal",
      fontWeightBold: "bold",
    });

    // 加载插件
    this.fitAddon = new FitAddon();
    // macOS 上禁用滚动条宽度计算（参考 waveterm）
    const isMac = /mac/i.test(navigator.userAgent);
    this.fitAddon.noScrollbar = isMac;

    this.searchAddon = new SearchAddon();
    const webLinksAddon = new WebLinksAddon();

    this.terminal.loadAddon(this.fitAddon);
    this.terminal.loadAddon(this.searchAddon);
    this.terminal.loadAddon(webLinksAddon);

    // 打开终端
    this.terminal.open(this.connectElem);

    // 设置防抖的 resize 处理（参考 waveterm 使用 50ms，增加到 100ms 以避免 TUI 初始化时的竞态）
    this.handleResize_debounced = debounce(this.handleResize.bind(this), 100);

    // 立即连接到会话
    this.connect();

    // 参考 waveterm：构造函数中调用 handleResize
    // 但需要等待 xterm 完全初始化（viewport 需要时间创建）
    this.handleResize();
  }

  /**
   * 连接到会话
   *
   * 设置输入处理和事件监听。
   */
  private async connect(): Promise<void> {
    if (this.connected) return;

    console.log(`[TermWrap] 连接到会话: ${this.sessionId}`);

    try {
      // 设置输入处理
      const onDataDisposable = this.terminal.onData((data) => {
        const base64 = encodeBase64(data);
        writeToTerminalRaw(this.sessionId, base64).catch(console.error);
      });
      this.toDispose.push(onDataDisposable);

      // 监听输出
      this.unlistenOutput = await onSessionOutput(this.sessionId, (data) => {
        this.terminal.write(decodeBytes(data));
      });

      // 监听状态
      this.unlistenStatus = await onSessionStatus(this.sessionId, (event) => {
        this.options.onStatusChange?.(event.status);
      });

      this.connected = true;
      console.log(`[TermWrap] 已连接到会话: ${this.sessionId}`);
    } catch (err) {
      console.error("[TermWrap] 连接失败:", err);
      this.options.onStatusChange?.("error");
    }
  }

  /**
   * 处理终端大小变化
   *
   * 调用 fitAddon.fit() 计算新大小，然后同步到后端。
   */
  handleResize(): void {
    // 安全检查：确保终端已初始化
    if (!this.terminal || !this.terminal.element) {
      console.log("[TermWrap] handleResize: 终端未初始化，跳过");
      return;
    }

    const oldRows = this.terminal.rows;
    const oldCols = this.terminal.cols;

    // 调用 fit 计算新大小
    try {
      this.fitAddon.fit();
    } catch (err) {
      console.warn("[TermWrap] fit() 失败:", err);
      return;
    }

    const { rows, cols } = this.terminal;

    console.log(
      `[TermWrap] handleResize: ${cols}x${rows} (was ${oldCols}x${oldRows})`,
    );

    // 同步大小到后端
    this.syncSizeToBackend(rows, cols);
  }

  /**
   * 同步大小到后端
   */
  private syncSizeToBackend(rows: number, cols: number): void {
    // 检查是否与上次同步的大小相同
    if (
      this.lastSyncedSize &&
      this.lastSyncedSize.rows === rows &&
      this.lastSyncedSize.cols === cols
    ) {
      return; // 大小没变，不需要同步
    }

    console.log(`[TermWrap] 同步 resize 到后端: ${cols}x${rows}`);
    this.lastSyncedSize = { rows, cols };
    resizeTerminal(this.sessionId, rows, cols).catch(console.error);
  }

  /**
   * 聚焦终端
   */
  focus(): void {
    this.terminal.focus();
  }

  // ============================================================================
  // 搜索功能
  // ============================================================================

  /**
   * 设置搜索回调
   */
  setSearchCallbacks(callbacks: SearchCallbacks): void {
    this.searchCallbacks = callbacks;
  }

  /**
   * 搜索文本
   * @param term - 搜索词
   * @param options - 搜索选项
   * @returns 是否找到匹配
   */
  search(term: string, options?: ISearchOptions): boolean {
    if (!term) {
      this.clearSearch();
      return false;
    }
    const found = this.searchAddon.findNext(term, options);
    return found;
  }

  /**
   * 搜索下一个
   */
  searchNext(term: string, options?: ISearchOptions): boolean {
    if (!term) return false;
    return this.searchAddon.findNext(term, options);
  }

  /**
   * 搜索上一个
   */
  searchPrevious(term: string, options?: ISearchOptions): boolean {
    if (!term) return false;
    return this.searchAddon.findPrevious(term, options);
  }

  /**
   * 清除搜索高亮
   */
  clearSearch(): void {
    this.searchAddon.clearDecorations();
  }

  // ============================================================================
  // 主题功能
  // ============================================================================

  /**
   * 设置主题
   */
  setTheme(themeName: ThemeName): void {
    this.currentTheme = themeName;
    const theme = getTheme(themeName);
    this.terminal.options.theme = theme;
  }

  /**
   * 获取当前主题名称
   */
  getThemeName(): ThemeName {
    return this.currentTheme;
  }

  /**
   * 销毁终端
   */
  dispose(): void {
    console.log(`[TermWrap] 销毁终端: ${this.sessionId}`);

    // 清理事件监听
    this.unlistenOutput?.();
    this.unlistenStatus?.();

    // 清理其他资源
    this.toDispose.forEach((d) => {
      try {
        d.dispose();
      } catch {
        // ignore dispose errors
      }
    });

    // 销毁终端
    this.terminal.dispose();

    this.connected = false;
  }
}
