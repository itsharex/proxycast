import { describe, expect, it } from "vitest";
import {
  estimateImageWorkbenchTaskCardHeight,
  resolveImageWorkbenchCanvasLayout,
  resolveImageWorkbenchFitViewport,
} from "./imageWorkbenchCanvasLayout";

describe("imageWorkbenchCanvasLayout", () => {
  it("应根据任务数量和容器宽度计算多列铺满布局", () => {
    const layout = resolveImageWorkbenchCanvasLayout({
      tasks: [
        { expectedCount: 4, outputCount: 2 },
        { expectedCount: 2, outputCount: 2 },
        { expectedCount: 1, outputCount: 1, hasFailureMessage: true },
      ],
      containerWidth: 1720,
      containerHeight: 980,
    });

    expect(layout.columns).toBe(3);
    expect(layout.cardWidth).toBeGreaterThanOrEqual(420);
    expect(layout.boardWidth).toBeGreaterThan(1200);
    expect(layout.surfaceWidth).toBeGreaterThanOrEqual(1720);
    expect(layout.surfaceHeight).toBeGreaterThanOrEqual(980);
  });

  it("应让失败任务卡高度高于普通任务", () => {
    const normalHeight = estimateImageWorkbenchTaskCardHeight({
      expectedCount: 2,
      outputCount: 2,
    });
    const failedHeight = estimateImageWorkbenchTaskCardHeight({
      expectedCount: 2,
      outputCount: 2,
      hasFailureMessage: true,
    });

    expect(failedHeight).toBeGreaterThan(normalHeight);
  });

  it("展开任务卡时应为内联详情预留更高画布高度", () => {
    const collapsedHeight = estimateImageWorkbenchTaskCardHeight({
      expectedCount: 2,
      outputCount: 2,
    });
    const expandedHeight = estimateImageWorkbenchTaskCardHeight({
      expectedCount: 2,
      outputCount: 2,
      expanded: true,
    });

    expect(expandedHeight).toBeGreaterThan(collapsedHeight);
  });

  it("应把内容居中并限制 fit scale 范围", () => {
    const viewport = resolveImageWorkbenchFitViewport({
      containerWidth: 1280,
      containerHeight: 820,
      boardWidth: 1680,
      boardHeight: 1040,
    });

    expect(viewport.scale).toBeGreaterThanOrEqual(0.55);
    expect(viewport.scale).toBeLessThanOrEqual(1.9);
    expect(viewport.x).toBeGreaterThan(0);
    expect(viewport.y).toBeGreaterThan(0);
  });
});
