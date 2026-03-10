import React from "react";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MarkdownRenderer } from "./MarkdownRenderer";

vi.mock("react-syntax-highlighter", () => ({
  Prism: ({
    children,
    className,
  }: {
    children: React.ReactNode;
    className?: string;
  }) => (
    <pre data-testid="syntax-highlighter" className={className}>
      {children}
    </pre>
  ),
}));

vi.mock("react-syntax-highlighter/dist/esm/styles/prism", () => ({
  oneDark: {},
}));

vi.mock("./ArtifactPlaceholder", () => ({
  ArtifactPlaceholder: ({ language }: { language: string }) => (
    <div data-testid="artifact-placeholder">{language}</div>
  ),
}));

vi.mock("./A2UITaskCard", () => ({
  A2UITaskCard: () => <div data-testid="a2ui-task-card" />,
  A2UITaskLoadingCard: () => <div data-testid="a2ui-task-loading-card" />,
}));

interface MountedHarness {
  container: HTMLDivElement;
  root: Root;
}

const mountedRoots: MountedHarness[] = [];

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
  vi.useRealTimers();
  vi.clearAllMocks();
});

function render(content: string, isStreaming = false): HTMLDivElement {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(
      <MarkdownRenderer content={content} isStreaming={isStreaming} />,
    );
  });

  mountedRoots.push({ container, root });
  return container;
}

function renderHarness(content: string, isStreaming = false) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  const rerender = (nextContent: string, nextIsStreaming = isStreaming) => {
    act(() => {
      root.render(
        <MarkdownRenderer
          content={nextContent}
          isStreaming={nextIsStreaming}
        />,
      );
    });
  };

  rerender(content, isStreaming);

  mountedRoots.push({ container, root });
  return { container, rerender };
}

describe("MarkdownRenderer", () => {
  it("非流式时应保留 raw html 渲染能力", () => {
    const content = [
      "前置文本",
      "",
      '<div class="rendered-html">原始 HTML</div>',
      "",
      "后置文本",
    ].join("\n");

    const container = render(content, false);

    expect(container.querySelector(".rendered-html")).not.toBeNull();
    expect(container.textContent).toContain("原始 HTML");
  });

  it("大段流式输出时应跳过 raw html 重解析", () => {
    const content = [
      "A".repeat(2_200),
      "",
      '<div class="rendered-html">原始 HTML</div>',
      "",
      "结尾文本",
    ].join("\n");

    const container = render(content, true);

    expect(container.querySelector(".rendered-html")).toBeNull();
    expect(container.textContent).toContain("结尾文本");
  });

  it("流式结束后应立即恢复完整 raw html 渲染", () => {
    vi.useFakeTimers();
    const content = [
      "A".repeat(2_200),
      "",
      '<div class="rendered-html">原始 HTML</div>',
      "",
      "结尾文本",
    ].join("\n");

    const { container, rerender } = renderHarness(content, true);
    expect(container.querySelector(".rendered-html")).toBeNull();

    rerender(content, false);
    expect(container.querySelector(".rendered-html")).not.toBeNull();
  });
});
