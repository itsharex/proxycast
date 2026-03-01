import { act } from "react";
import type { ReactNode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanupMountedRoots,
  flushEffects,
  renderIntoDom,
  setReactActEnvironment,
  waitForCondition,
  type MountedRoot,
} from "../test-utils";

const { mockInvoke, mockToastSuccess, mockToastError } = vi.hoisted(() => ({
  mockInvoke: vi.fn(),
  mockToastSuccess: vi.fn(),
  mockToastError: vi.fn(),
}));

const {
  mockEmitCanvasImageInsertRequest,
  mockOnCanvasImageInsertAck,
  mockGetActiveContentTarget,
} = vi.hoisted(() => ({
  mockEmitCanvasImageInsertRequest: vi.fn(),
  mockOnCanvasImageInsertAck: vi.fn(),
  mockGetActiveContentTarget: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@/components/ui/scroll-area", () => ({
  ScrollArea: ({ children }: { children: ReactNode }) => (
    <div data-testid="scroll-area">{children}</div>
  ),
}));

vi.mock("sonner", () => ({
  toast: {
    success: mockToastSuccess,
    error: mockToastError,
  },
}));

vi.mock("@/lib/canvasImageInsertBus", () => ({
  emitCanvasImageInsertRequest: mockEmitCanvasImageInsertRequest,
  onCanvasImageInsertAck: mockOnCanvasImageInsertAck,
}));

vi.mock("@/lib/activeContentTarget", () => ({
  getActiveContentTarget: mockGetActiveContentTarget,
}));

import { ImageSearchTab } from "./ImageSearchTab";

const mountedRoots: MountedRoot[] = [];

function renderTab(projectId = "project-1"): HTMLDivElement {
  const mounted = renderIntoDom(
    <ImageSearchTab projectId={projectId} />,
    mountedRoots,
  );
  return mounted.container;
}

function findTextarea(container: HTMLElement): HTMLTextAreaElement {
  const node = container.querySelector("textarea");
  if (!node) {
    throw new Error("未找到搜索输入框");
  }
  return node as HTMLTextAreaElement;
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

async function setInputValue(input: HTMLTextAreaElement, value: string) {
  const nativeSetter = Object.getOwnPropertyDescriptor(
    window.HTMLTextAreaElement.prototype,
    "value",
  )?.set;
  if (!nativeSetter) {
    throw new Error("未找到 textarea value setter");
  }

  await act(async () => {
    nativeSetter.call(input, value);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    await flushEffects();
  });
}

beforeEach(() => {
  setReactActEnvironment();
  vi.clearAllMocks();
  vi.stubGlobal("open", vi.fn());
  mockGetActiveContentTarget.mockReturnValue({
    projectId: "project-1",
    contentId: "content-1",
    canvasType: "document",
  });
  mockOnCanvasImageInsertAck.mockReturnValue(() => undefined);
  mockEmitCanvasImageInsertRequest.mockReturnValue({
    requestId: "insert-1",
  });

  mockInvoke.mockImplementation((command, payload) => {
    if (command === "search_pixabay_images") {
      const page = payload.req.page;
      return Promise.resolve({
        total: 40,
        total_hits: 40,
        hits: [
          {
            id: page,
            preview_url: `https://pixabay.example/${page}-preview.jpg`,
            large_image_url: `https://pixabay.example/${page}-large.jpg`,
            image_width: 1200,
            image_height: 800,
            tags: `pixabay-${page}`,
            page_url: `https://pixabay.com/photos/${page}`,
            user: "pixabay-user",
          },
        ],
      });
    }

    if (command === "search_web_images") {
      const page = payload.req.page;
      return Promise.resolve({
        total: 40,
        provider: "pexels",
        hits: [
          {
            id: `w-${page}`,
            thumbnail_url: `https://pexels.example/${page}-thumb.jpg`,
            content_url: `https://pexels.example/${page}-image.jpg`,
            width: 1080,
            height: 1920,
            name: `pexels-${page}`,
            host_page_url: `https://www.pexels.com/photo/${page}`,
          },
        ],
      });
    }

    if (command === "import_material_from_url") {
      return Promise.resolve({ id: "mock-material-id" });
    }

    return Promise.reject(new Error(`unexpected command: ${command}`));
  });
});

afterEach(() => {
  cleanupMountedRoots(mountedRoots);
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("ImageSearchTab", () => {
  it("应支持切换来源并展示对应 attribution", async () => {
    const container = renderTab();
    await setInputValue(findTextarea(container), "city");

    await act(async () => {
      findButton(container, "搜索图片").click();
      await flushEffects();
    });

    await waitForCondition(
      () => container.textContent?.includes("图片来源: Pexels") ?? false,
      50,
      "未展示 Pexels 来源",
    );

    await act(async () => {
      findButton(container, "Pixabay图库").click();
    });

    await act(async () => {
      findButton(container, "搜索图片").click();
      await flushEffects();
    });

    await waitForCondition(
      () => container.textContent?.includes("图片来源: Pixabay") ?? false,
      50,
      "未展示 Pixabay 来源",
    );

    expect(mockInvoke).toHaveBeenCalledWith(
      "search_web_images",
      expect.objectContaining({
        req: expect.objectContaining({
          query: "city",
          page: 1,
        }),
      }),
    );
  });

  it("在联网来源保存图片时应使用 pexels 标签", async () => {
    const container = renderTab();
    await setInputValue(findTextarea(container), "city");

    await act(async () => {
      findButton(container, "联网搜索").click();
    });

    await act(async () => {
      findButton(container, "搜索图片").click();
      await flushEffects();
    });

    await waitForCondition(
      () => container.textContent?.includes("保存") ?? false,
      50,
      "搜索结果未渲染保存按钮",
    );

    await act(async () => {
      findButton(container, "保存").click();
      await flushEffects();
    });

    expect(mockInvoke).toHaveBeenCalledWith(
      "import_material_from_url",
      expect.objectContaining({
        req: expect.objectContaining({
          projectId: "project-1",
          type: "image",
          tags: ["pexels"],
        }),
      }),
    );
    expect(mockToastSuccess).toHaveBeenCalled();
  });

  it("当前来源加载更多应请求下一页", async () => {
    const container = renderTab();
    await setInputValue(findTextarea(container), "beach");

    await act(async () => {
      findButton(container, "联网搜索").click();
      await flushEffects();
    });

    await act(async () => {
      findButton(container, "搜索图片").click();
      await flushEffects();
    });

    await waitForCondition(
      () => container.textContent?.includes("加载更多") ?? false,
      50,
      "未出现加载更多按钮",
    );

    await act(async () => {
      findButton(container, "加载更多").click();
      await flushEffects();
    });

    const webSearchPages = mockInvoke.mock.calls
      .filter(([command]) => command === "search_web_images")
      .map(([, payload]) => payload.req.page);
    expect(webSearchPages).toEqual([1, 2]);
  });

  it("应支持将搜索图片插入当前画布", async () => {
    const container = renderTab();
    await setInputValue(findTextarea(container), "city");

    await act(async () => {
      findButton(container, "搜索图片").click();
      await flushEffects();
    });

    await waitForCondition(
      () => container.textContent?.includes("插入当前画布") ?? false,
      50,
      "搜索结果未渲染插入按钮",
    );

    await act(async () => {
      findButton(container, "插入当前画布").click();
      await flushEffects();
    });

    expect(mockEmitCanvasImageInsertRequest).toHaveBeenCalledWith(
      expect.objectContaining({
        projectId: "project-1",
        contentId: "content-1",
        canvasType: "document",
        source: "pexels",
        image: expect.objectContaining({
          contentUrl: "https://pexels.example/1-image.jpg",
          attributionName: "Pexels",
        }),
      }),
    );
    expect(mockToastSuccess).toHaveBeenCalled();
  });
});
