import { act } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  cleanupMountedRoots,
  clickButtonByText,
  clickByTestId,
  mountHarness,
  setupReactActEnvironment,
  type MountedRoot,
} from "../hooks/testUtils";
import {
  GeneratedOutputsPanel,
  type GeneratedOutputItem,
} from "./workbenchRightRailGeneratedOutputs";

setupReactActEnvironment();

describe("GeneratedOutputsPanel", () => {
  const mountedRoots: MountedRoot[] = [];

  afterEach(() => {
    cleanupMountedRoots(mountedRoots);
  });

  it("应只突出最近任务，并在展开后显示更早记录", () => {
    const latestAction = vi.fn();
    const olderAction = vi.fn();
    const items: GeneratedOutputItem[] = [
      {
        id: "latest",
        title: "图片任务已提交",
        detail: "基础模型 · 16:9 · 城市夜景",
        assetType: "image",
        assetUrl: "https://example.com/latest.png",
        actionLabel: "查看画布",
        onAction: latestAction,
      },
      {
        id: "older",
        title: "封面任务已提交",
        detail: "小红书 · 9:16 · 2 张",
        actionLabel: "查看画布",
        onAction: olderAction,
      },
    ];

    const { container } = mountHarness(
      GeneratedOutputsPanel,
      { items },
      mountedRoots,
    );

    expect(container.textContent).toContain("最近任务");
    expect(
      container.querySelector("[data-testid='workbench-generated-output-image']"),
    ).not.toBeNull();
    expect(
      container.querySelector("[data-testid='workbench-generated-output-history']"),
    ).toBeNull();
    expect(container.textContent).toContain("更早 1 条");

    clickButtonByText(container, "查看画布", { exact: true });
    expect(latestAction).toHaveBeenCalledTimes(1);

    clickByTestId(container, "workbench-generated-output-history-toggle");
    expect(
      container.querySelector("[data-testid='workbench-generated-output-history']"),
    ).not.toBeNull();
    expect(container.textContent).toContain("封面任务已提交");

    const olderButton = Array.from(container.querySelectorAll("button")).filter(
      (button) => button.textContent?.trim() === "查看画布",
    )[1];
    act(() => {
      olderButton?.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    });
    expect(olderAction).toHaveBeenCalledTimes(1);
  });
});
