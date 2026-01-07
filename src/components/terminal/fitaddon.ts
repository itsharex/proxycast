/**
 * @file fitaddon.ts
 * @description 自定义 FitAddon - 复制自 waveterm
 * @module components/terminal/fitaddon
 *
 * 修改自 xterm.js 官方 FitAddon，主要改动：
 * - 在 resize 前清除渲染服务
 * - 支持禁用滚动条宽度计算（macOS）
 */

import type { ITerminalAddon, Terminal } from "@xterm/xterm";

interface ITerminalDimensions {
  rows: number;
  cols: number;
}

const MINIMUM_COLS = 2;
const MINIMUM_ROWS = 1;

export class FitAddon implements ITerminalAddon {
  private _terminal: Terminal | undefined;
  /** 是否禁用滚动条宽度计算（macOS 上设为 true） */
  public noScrollbar: boolean = false;

  public activate(terminal: Terminal): void {
    this._terminal = terminal;
  }

  public dispose(): void {}

  public fit(): void {
    const dims = this.proposeDimensions();
    if (!dims || !this._terminal || isNaN(dims.cols) || isNaN(dims.rows)) {
      return;
    }

    // 访问 xterm 内部 API
    const core = (this._terminal as any)._core;

    // 安全检查：确保内部 API 可用
    if (!core || !core._renderService) {
      return;
    }

    // 关键：如果大小变化，先清除渲染再 resize
    if (
      this._terminal.rows !== dims.rows ||
      this._terminal.cols !== dims.cols
    ) {
      core._renderService.clear();
      this._terminal.resize(dims.cols, dims.rows);
    }
  }

  public proposeDimensions(): ITerminalDimensions | undefined {
    if (!this._terminal) {
      return undefined;
    }

    if (!this._terminal.element || !this._terminal.element.parentElement) {
      return undefined;
    }

    const core = (this._terminal as any)._core;

    if (!core || !core._renderService) {
      return undefined;
    }

    const dims = core._renderService.dimensions;

    if (
      !dims ||
      !dims.css ||
      !dims.css.cell ||
      dims.css.cell.width === 0 ||
      dims.css.cell.height === 0
    ) {
      return undefined;
    }

    // 计算滚动条宽度
    let scrollbarWidth = 0;
    if (
      core.viewport &&
      core.viewport._viewportElement &&
      core.viewport._scrollArea
    ) {
      const measuredScrollBarWidth =
        core.viewport._viewportElement.offsetWidth -
        core.viewport._scrollArea.offsetWidth;
      scrollbarWidth =
        this._terminal.options.scrollback === 0 ? 0 : measuredScrollBarWidth;
    }
    if (this.noScrollbar) {
      scrollbarWidth = 0;
    }

    // Use getBoundingClientRect for more accurate measurement including fractional pixels
    const parentRect =
      this._terminal.element.parentElement.getBoundingClientRect();
    const parentElementHeight = parentRect.height;
    const parentElementWidth = parentRect.width;

    console.log("[FitAddon Debug] Measuring parent (Rect):", {
      width: parentElementWidth,
      height: parentElementHeight,
      top: parentRect.top,
      bottom: parentRect.bottom,
    });

    const elementStyle = window.getComputedStyle(this._terminal.element);
    const elementPadding = {
      top: parseInt(elementStyle.getPropertyValue("padding-top")) || 0,
      bottom: parseInt(elementStyle.getPropertyValue("padding-bottom")) || 0,
      right: parseInt(elementStyle.getPropertyValue("padding-right")) || 0,
      left: parseInt(elementStyle.getPropertyValue("padding-left")) || 0,
    };

    const elementPaddingVer = elementPadding.top + elementPadding.bottom;
    const elementPaddingHor = elementPadding.right + elementPadding.left;

    const availableHeight = parentElementHeight - elementPaddingVer;
    const availableWidth =
      parentElementWidth - elementPaddingHor - scrollbarWidth;

    const geometry = {
      cols: Math.max(
        MINIMUM_COLS,
        Math.floor(availableWidth / dims.css.cell.width),
      ),
      rows: Math.max(
        MINIMUM_ROWS,
        Math.floor(availableHeight / dims.css.cell.height),
      ),
    };

    console.log("[FitAddon] proposeDimensions:", {
      parentElementHeight,
      parentElementWidth,
      availableHeight,
      availableWidth,
      cellHeight: dims.css.cell.height,
      cellWidth: dims.css.cell.width,
      geometry,
    });

    return geometry;
  }
}
