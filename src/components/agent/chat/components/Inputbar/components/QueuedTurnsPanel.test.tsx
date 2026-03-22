import React from "react";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { QueuedTurnsPanel } from "./QueuedTurnsPanel";

const mountedRoots: Array<{ root: Root; container: HTMLDivElement }> = [];

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

afterEach(() => {
  while (mountedRoots.length > 0) {
    const mounted = mountedRoots.pop();
    if (!mounted) break;
    act(() => {
      mounted.root.unmount();
    });
    mounted.container.remove();
  }
  vi.clearAllMocks();
});

function renderQueuedTurnsPanel(
  props?: Partial<React.ComponentProps<typeof QueuedTurnsPanel>>,
) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <QueuedTurnsPanel
        queuedTurns={[
          {
            queued_turn_id: "queued-1",
            message_preview: "等待整理周报",
            message_text: "请先整理周报结构，再补齐摘要。",
            created_at: 1_700_000_000_000,
            image_count: 0,
            position: 1,
          },
        ]}
        {...props}
      />,
    );
  });

  mountedRoots.push({ root, container });
  return container;
}

describe("QueuedTurnsPanel", () => {
  it("应展示立即执行按钮，并触发 promote 回调", async () => {
    const onPromoteQueuedTurn = vi.fn().mockResolvedValue(true);
    const container = renderQueuedTurnsPanel({ onPromoteQueuedTurn });

    expect(container.textContent).toContain("稍后处理 1");
    expect(container.textContent).toContain("会依次开始");

    const promoteButton = Array.from(
      container.querySelectorAll<HTMLButtonElement>("button"),
    ).find((button) => button.textContent?.includes("立即执行"));

    expect(promoteButton).toBeTruthy();

    await act(async () => {
      promoteButton?.click();
      await Promise.resolve();
    });

    expect(onPromoteQueuedTurn).toHaveBeenCalledWith("queued-1");
  });

  it("移除按钮仍应触发 remove 回调", async () => {
    const onRemoveQueuedTurn = vi.fn().mockResolvedValue(true);
    const container = renderQueuedTurnsPanel({ onRemoveQueuedTurn });

    const removeButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="移除排队消息"]',
    );

    expect(removeButton).toBeTruthy();

    await act(async () => {
      removeButton?.click();
      await Promise.resolve();
    });

    expect(onRemoveQueuedTurn).toHaveBeenCalledWith("queued-1");
  });
});
