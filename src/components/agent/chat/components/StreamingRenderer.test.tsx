import React from "react";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { StreamingRenderer } from "./StreamingRenderer";
import type { ContentPart } from "../types";

const parseAIResponseMock = vi.fn();

vi.mock("@/components/content-creator/a2ui/parser", () => ({
  parseAIResponse: (...args: unknown[]) => parseAIResponseMock(...args),
}));

vi.mock("@/lib/artifact/hooks/useDebouncedValue", () => ({
  useDebouncedValue: <T,>(value: T) => value,
}));

vi.mock("./MarkdownRenderer", () => ({
  MarkdownRenderer: ({ content }: { content: string }) => (
    <div data-testid="markdown-renderer">{content}</div>
  ),
}));

vi.mock("./A2UITaskCard", () => ({
  A2UITaskCard: () => <div data-testid="a2ui-card" />,
  A2UITaskLoadingCard: () => <div data-testid="a2ui-loading-card" />,
}));

vi.mock("./ToolCallDisplay", () => ({
  ToolCallList: () => <div data-testid="tool-call-list" />,
  ToolCallItem: () => <div data-testid="tool-call-item" />,
}));

vi.mock("./DecisionPanel", () => ({
  DecisionPanel: () => <div data-testid="decision-panel" />,
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
  parseAIResponseMock.mockImplementation((content: string) => ({
    parts: content.trim() ? [{ type: "text", content: content.trim() }] : [],
    hasA2UI: false,
    hasWriteFile: false,
    hasPending: false,
  }));
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

function renderHarness(props: {
  content: string;
  isStreaming?: boolean;
  contentParts?: ContentPart[];
}) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  const rerender = (nextProps: typeof props) => {
    act(() => {
      root.render(<StreamingRenderer {...nextProps} />);
    });
  };

  rerender(props);
  mountedRoots.push({ container, root });

  return { container, rerender };
}

describe("StreamingRenderer", () => {
  it("纯文本内容应短路跳过结构化解析", () => {
    renderHarness({
      content: "这是普通文本输出，不包含结构化标签。",
      isStreaming: true,
    });

    expect(parseAIResponseMock).not.toHaveBeenCalled();
  });

  it("交错内容重复渲染时应复用已缓存解析结果", () => {
    const structuredText = '<write_file path="demo.md">hello</write_file>';
    parseAIResponseMock.mockImplementation((content: string) => {
      if (content === structuredText) {
        return {
          parts: [
            {
              type: "write_file",
              content: "hello",
              filePath: "demo.md",
            },
          ],
          hasA2UI: false,
          hasWriteFile: true,
          hasPending: false,
        };
      }

      return {
        parts: content.trim()
          ? [{ type: "text", content: content.trim() }]
          : [],
        hasA2UI: false,
        hasWriteFile: false,
        hasPending: false,
      };
    });
    const contentParts: ContentPart[] = [
      { type: "text", text: structuredText },
      { type: "text", text: "普通文本" },
    ];

    const { rerender } = renderHarness({
      content: structuredText,
      contentParts,
      isStreaming: true,
    });

    expect(parseAIResponseMock).toHaveBeenCalledTimes(1);

    rerender({
      content: structuredText,
      contentParts: [...contentParts],
      isStreaming: true,
    });

    expect(parseAIResponseMock).toHaveBeenCalledTimes(1);
  });
});
