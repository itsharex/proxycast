import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ToolCallDisplay, ToolCallList } from "./ToolCallDisplay";
import type { ToolCallState } from "@/lib/api/agentStream";

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn().mockResolvedValue(undefined),
}));

interface RenderResult {
  container: HTMLDivElement;
  root: Root;
}

const mountedRoots: RenderResult[] = [];

function renderTool(toolCall: ToolCallState): RenderResult {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  act(() => {
    root.render(<ToolCallDisplay toolCall={toolCall} />);
  });

  const rendered = { container, root };
  mountedRoots.push(rendered);
  return rendered;
}

afterEach(() => {
  while (mountedRoots.length > 0) {
    const mounted = mountedRoots.pop();
    if (!mounted) {
      break;
    }
    act(() => {
      mounted.root.unmount();
    });
    mounted.container.remove();
  }
});

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;
});

describe("ToolCallDisplay", () => {
  it("WebSearch 工具结果应在 AI 对话区展示搜索列表并支持悬浮预览", async () => {
    renderTool({
      id: "tool-search-1",
      name: "WebSearch",
      arguments: JSON.stringify({ query: "3月13日国际新闻" }),
      status: "completed",
      result: {
        success: true,
        output: [
          "Xinhua world news summary at 0030 GMT, March 13",
          "https://example.com/xinhua",
          "全球要闻摘要，覆盖国际局势与市场动态。",
          "",
          "Friday morning news: March 13, 2026 | WORLD - wng.org",
          "https://example.com/wng",
          "补充国际动态与区域冲突更新。",
        ].join("\n"),
      },
      startTime: new Date("2026-03-13T12:00:00.000Z"),
      endTime: new Date("2026-03-13T12:00:02.000Z"),
    });

    expect(document.body.textContent).toContain(
      "Xinhua world news summary at 0030 GMT, March 13",
    );
    expect(document.body.textContent).toContain(
      "Friday morning news: March 13, 2026 | WORLD - wng.org",
    );

    const firstSearchResult = document.body.querySelector(
      '[aria-label="预览搜索结果：Xinhua world news summary at 0030 GMT, March 13"]',
    ) as HTMLButtonElement | null;

    await act(async () => {
      firstSearchResult?.dispatchEvent(
        new MouseEvent("mouseover", { bubbles: true }),
      );
      await Promise.resolve();
    });

    expect(document.body.textContent).toContain(
      "全球要闻摘要，覆盖国际局势与市场动态。",
    );
    expect(document.body.textContent).toContain("https://example.com/xinhua");

    const collapseButton = document.body.querySelector(
      'button[title="收起结果"]',
    ) as HTMLButtonElement | null;

    act(() => {
      collapseButton?.click();
    });

    expect(document.body.textContent).not.toContain(
      "Xinhua world news summary at 0030 GMT, March 13",
    );

    const expandButton = document.body.querySelector(
      'button[title="查看结果"]',
    ) as HTMLButtonElement | null;

    act(() => {
      expandButton?.click();
    });

    expect(document.body.textContent).toContain(
      "Xinhua world news summary at 0030 GMT, March 13",
    );
  });

  it("连续多次 WebSearch 应在对话区按搜索批次分组展示", () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <ToolCallList
          toolCalls={[
            {
              id: "tool-search-1",
              name: "WebSearch",
              arguments: JSON.stringify({ query: "3月13日国际新闻" }),
              status: "completed",
              result: { success: true, output: "https://example.com/1" },
              startTime: new Date("2026-03-13T12:00:00.000Z"),
              endTime: new Date("2026-03-13T12:00:01.000Z"),
            },
            {
              id: "tool-search-2",
              name: "WebSearch",
              arguments: JSON.stringify({ query: "March 13 2026 world headlines" }),
              status: "completed",
              result: { success: true, output: "https://example.com/2" },
              startTime: new Date("2026-03-13T12:00:02.000Z"),
              endTime: new Date("2026-03-13T12:00:03.000Z"),
            },
          ]}
        />,
      );
    });

    mountedRoots.push({ container, root });

    expect(container.textContent).toContain("已搜索");
    expect(container.textContent).toContain("2");
    expect(container.textContent).toContain("3月13日国际新闻");
    expect(container.textContent).toContain("March 13 2026 world headlines");
    expect(container.textContent).toContain("搜索 3月13日国际新闻");
    expect(container.textContent).toContain("搜索 March 13 2026 world headlines");
    expect(container.textContent).toContain("中文日期检索");
    expect(container.textContent).toContain("头条检索");
  });

  it("连续完成的命令工具应聚合成一个 work group", () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <ToolCallList
          toolCalls={[
            {
              id: "tool-exec-1",
              name: "bash",
              arguments: JSON.stringify({ command: "pwd" }),
              status: "completed",
              result: { success: true, output: "/workspace\n" },
              startTime: new Date("2026-03-20T12:00:00.000Z"),
              endTime: new Date("2026-03-20T12:00:01.000Z"),
            },
            {
              id: "tool-exec-2",
              name: "bash",
              arguments: JSON.stringify({ command: "ls -la" }),
              status: "completed",
              result: { success: true, output: "file-a\nfile-b\n" },
              startTime: new Date("2026-03-20T12:00:02.000Z"),
              endTime: new Date("2026-03-20T12:00:03.000Z"),
            },
          ]}
        />,
      );
    });

    mountedRoots.push({ container, root });

    const groups = container.querySelectorAll('[data-testid="tool-call-work-group"]');
    expect(groups).toHaveLength(1);
    expect(container.textContent).toContain("已执行 2 条命令");
    expect(container.textContent).toContain("2");
    expect(container.textContent).toContain("pwd");
    expect(container.textContent).toContain("ls -la");

    act(() => {
      const groupToggle = groups[0]?.querySelector("button") as HTMLButtonElement | null;
      groupToggle?.click();
    });

    expect(container.textContent).toContain("执行 pwd");
    expect(container.textContent).toContain("执行 ls -la");
    expect(container.textContent).not.toContain("pwd · ls -la");
  });

  it("命令结果应进入代码块渲染，而不是裸文本标题重复", () => {
    const { container } = renderTool({
      id: "tool-exec-render-1",
      name: "bash",
      arguments: JSON.stringify({ command: "ls -la" }),
      status: "completed",
      result: {
        success: true,
        output: "/tmp\nfile-a\nfile-b\nfile-c\n",
        metadata: {
          exit_code: 0,
          stdout_length: 24,
          stderr_length: 0,
        },
      },
      startTime: new Date("2026-03-20T12:10:00.000Z"),
      endTime: new Date("2026-03-20T12:10:01.000Z"),
    });

    act(() => {
      const toggle = container.querySelector(
        'button[title="查看结果"]',
      ) as HTMLButtonElement | null;
      toggle?.click();
    });

    expect(container.textContent).toContain("已执行 ls -la");
    expect(container.textContent).not.toContain("已执行已执行");
    expect(
      container.querySelector('[data-testid="tool-call-rendered-result"]'),
    ).toBeTruthy();
    expect(container.textContent).toContain("text");
    expect(container.textContent).toContain("Copy");
  });

  it("应为浏览器、委派、任务输出与交互类工具生成具体动作句", () => {
    const container = document.createElement("div");
    document.body.appendChild(container);
    const root = createRoot(container);

    act(() => {
      root.render(
        <ToolCallList
          toolCalls={[
            {
              id: "tool-browser-1",
              name: "mcp__lime-browser__browser_navigate",
              arguments: JSON.stringify({ url: "https://example.com/docs" }),
              status: "completed",
              result: { success: true, output: "ok" },
              startTime: new Date("2026-03-20T12:20:00.000Z"),
              endTime: new Date("2026-03-20T12:20:01.000Z"),
            },
            {
              id: "tool-subagent-1",
              name: "spawn_agent",
              arguments: JSON.stringify({ description: "修复登录页" }),
              status: "running",
              startTime: new Date("2026-03-20T12:20:02.000Z"),
            },
            {
              id: "tool-output-1",
              name: "TaskOutput",
              arguments: JSON.stringify({ task_id: "video-task-1" }),
              status: "completed",
              result: { success: true, output: "done" },
              startTime: new Date("2026-03-20T12:20:03.000Z"),
              endTime: new Date("2026-03-20T12:20:04.000Z"),
            },
            {
              id: "tool-skill-1",
              name: "load_skill",
              arguments: JSON.stringify({ name: "lime-governance" }),
              status: "completed",
              result: { success: true, output: "loaded" },
              startTime: new Date("2026-03-20T12:20:05.000Z"),
              endTime: new Date("2026-03-20T12:20:06.000Z"),
            },
            {
              id: "tool-glob-1",
              name: "glob",
              arguments: JSON.stringify({ pattern: "src/**/*.tsx" }),
              status: "completed",
              result: { success: true, output: "matched" },
              startTime: new Date("2026-03-20T12:20:07.000Z"),
              endTime: new Date("2026-03-20T12:20:08.000Z"),
            },
            {
              id: "tool-input-1",
              name: "request_user_input",
              arguments: JSON.stringify({ question: "需要继续吗？" }),
              status: "running",
              startTime: new Date("2026-03-20T12:20:09.000Z"),
            },
          ]}
        />,
      );
    });

    mountedRoots.push({ container, root });

    expect(container.textContent).toContain("已打开 https://example.com/docs");
    expect(container.textContent).toContain("协作中 修复登录页");
    expect(container.textContent).toContain("已读取输出 video-task-1");
    expect(container.textContent).toContain("已加载技能 lime-governance");
    expect(container.textContent).toContain("已列出 src/**/*.tsx");
    expect(container.textContent).toContain("等待输入 需要继续吗？");
  });
});
