import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { mockGetConfig, mockSaveConfig } = vi.hoisted(() => ({
  mockGetConfig: vi.fn(),
  mockSaveConfig: vi.fn(),
}));
const { mockOpen } = vi.hoisted(() => ({
  mockOpen: vi.fn(),
}));

vi.mock("@/hooks/useTauri", () => ({
  getConfig: mockGetConfig,
  saveConfig: mockSaveConfig,
}));
vi.mock("@tauri-apps/plugin-shell", () => ({
  open: mockOpen,
}));

import { WebSearchSettings } from ".";

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
    root.render(<WebSearchSettings />);
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

function findSelect(container: HTMLElement, id: string): HTMLSelectElement {
  const node = container.querySelector<HTMLSelectElement>(`#${id}`);
  if (!node) {
    throw new Error(`未找到下拉框: ${id}`);
  }
  return node;
}

function findInput(container: HTMLElement, id: string): HTMLInputElement {
  const node = container.querySelector<HTMLInputElement>(`#${id}`);
  if (!node) {
    throw new Error(`未找到输入框: ${id}`);
  }
  return node;
}

async function setInputValue(input: HTMLInputElement, value: string) {
  const nativeSetter = Object.getOwnPropertyDescriptor(
    window.HTMLInputElement.prototype,
    "value",
  )?.set;
  if (!nativeSetter) {
    throw new Error("未找到 input value setter");
  }

  await act(async () => {
    nativeSetter.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await flushEffects();
  });
}

async function setSelectValue(select: HTMLSelectElement, value: string) {
  const nativeSetter = Object.getOwnPropertyDescriptor(
    window.HTMLSelectElement.prototype,
    "value",
  )?.set;
  if (!nativeSetter) {
    throw new Error("未找到 select value setter");
  }

  await act(async () => {
    nativeSetter.call(select, value);
    select.dispatchEvent(new Event("change", { bubbles: true }));
    await flushEffects();
  });
}

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;

  vi.clearAllMocks();

  mockGetConfig.mockResolvedValue({
    web_search: {
      engine: "google",
    },
    image_gen: {
      image_search_pexels_api_key: "old-key",
      image_search_pixabay_api_key: "old-pixabay-key",
    },
  });
  mockSaveConfig.mockResolvedValue(undefined);
  mockOpen.mockResolvedValue(undefined);
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

describe("WebSearchSettings", () => {
  it("应加载网络搜索与图片搜索配置", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    const select = findSelect(container, "web-search-engine");
    expect(select.value).toBe("google");

    const input = findInput(container, "web-search-pexels-key");
    expect(input.value).toBe("old-key");
    const pixabayInput = findInput(container, "web-search-pixabay-key");
    expect(pixabayInput.value).toBe("old-pixabay-key");
  });

  it("修改搜索引擎和图片 Key 后应统一保存", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    await setSelectValue(
      findSelect(container, "web-search-engine"),
      "xiaohongshu",
    );
    await setInputValue(
      findInput(container, "web-search-pexels-key"),
      "new-key",
    );
    await setInputValue(
      findInput(container, "web-search-pixabay-key"),
      "new-pixabay-key",
    );

    await act(async () => {
      findButton(container, "保存").click();
      await flushEffects();
    });

    expect(mockSaveConfig).toHaveBeenCalledTimes(1);
    expect(mockSaveConfig).toHaveBeenCalledWith(
      expect.objectContaining({
        web_search: {
          engine: "xiaohongshu",
        },
        image_gen: expect.objectContaining({
          image_search_pexels_api_key: "new-key",
          image_search_pixabay_api_key: "new-pixabay-key",
        }),
      }),
    );
    expect(container.textContent).toContain("网络搜索设置已保存");
  });

  it("点击一键申请 Key 应打开官方申请页面", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    await act(async () => {
      findButton(container, "申请 Pexels Key").click();
      await flushEffects();
    });

    expect(mockOpen).toHaveBeenCalledWith("https://www.pexels.com/api/new/");
  });

  it("插件打开失败时应回退到 window.open", async () => {
    mockOpen.mockRejectedValueOnce(new Error("plugin failed"));
    const fallbackSpy = vi.spyOn(window, "open").mockImplementation(() => null);

    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    await act(async () => {
      findButton(container, "申请 Pexels Key").click();
      await flushEffects();
    });

    expect(fallbackSpy).toHaveBeenCalledWith(
      "https://www.pexels.com/api/new/",
      "_blank",
    );
  });

  it("点击 Pixabay 申请按钮应打开官方页面", async () => {
    const container = renderComponent();
    await flushEffects();
    await flushEffects();

    await act(async () => {
      findButton(container, "申请 Pixabay Key").click();
      await flushEffects();
    });

    expect(mockOpen).toHaveBeenCalledWith("https://pixabay.com/accounts/register/");
  });
});
