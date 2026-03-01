import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { mockGetConfig, mockSaveConfig } = vi.hoisted(() => ({
  mockGetConfig: vi.fn(),
  mockSaveConfig: vi.fn(),
}));

vi.mock("@/hooks/useTauri", () => ({
  getConfig: mockGetConfig,
  saveConfig: mockSaveConfig,
}));

import { ImageGenSettings } from ".";

interface Mounted {
  container: HTMLDivElement;
  root: Root;
}

const mounted: Mounted[] = [];

function renderComponent(): HTMLDivElement {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);
  act(() => {
    root.render(<ImageGenSettings />);
  });
  mounted.push({ container, root });
  return container;
}

async function flushEffects() {
  await act(async () => {
    await Promise.resolve();
  });
}

function findButton(container: HTMLElement, text: string): HTMLButtonElement {
  const target = Array.from(container.querySelectorAll("button")).find(
    (button) => button.textContent?.includes(text),
  );
  if (!target) {
    throw new Error(`未找到按钮: ${text}`);
  }
  return target as HTMLButtonElement;
}

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;

  vi.clearAllMocks();

  mockGetConfig.mockResolvedValue({
    image_gen: {
      default_service: "dall_e",
      default_count: 1,
      default_size: "1024x1024",
      default_quality: "standard",
      default_style: "vivid",
      enable_enhancement: false,
      auto_download: false,
      image_search_pexels_api_key: "old-key",
    },
  });
  mockSaveConfig.mockResolvedValue(undefined);
});

afterEach(() => {
  while (mounted.length > 0) {
    const target = mounted.pop();
    if (!target) break;
    act(() => {
      target.root.unmount();
    });
    target.container.remove();
  }
  vi.clearAllMocks();
});

describe("ImageGenSettings", () => {
  it("应加载图像生成配置", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    expect(container.textContent).toContain("默认图像生成服务");
    expect(container.textContent).toContain("默认图像数量");
  });

  it("修改默认图像数量后应调用保存配置", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    await act(async () => {
      findButton(container, "3").click();
      await flushEffects();
    });

    expect(mockSaveConfig).toHaveBeenCalledTimes(1);
    expect(mockSaveConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        image_gen: expect.objectContaining({
          default_count: 3,
        }),
      }),
    );
    expect(container.textContent).toContain("设置已保存");
  });
});
