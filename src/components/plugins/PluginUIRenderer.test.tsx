/**
 * @file PluginUIRenderer 单元测试
 * @description 测试插件 UI 渲染器组件
 * @module components/plugins/PluginUIRenderer.test
 *
 * _需求: 3.2_
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import React, { act } from "react";
import { createRoot } from "react-dom/client";
import { PluginUIRenderer, type Page } from "./PluginUIRenderer";

// Mock MachineIdTool 组件
vi.mock("@/components/tools/machine-id/MachineIdTool", () => ({
  MachineIdTool: (_props: { onNavigate: (page: Page) => void }) => (
    <div data-testid="machine-id-tool">MachineIdTool Mock</div>
  ),
}));

// Mock lucide-react icons
vi.mock("lucide-react", () => ({
  AlertCircle: () => <span data-testid="alert-circle-icon">AlertCircle</span>,
  Package: () => <span data-testid="package-icon">Package</span>,
}));

describe("PluginUIRenderer", () => {
  const mockNavigate = vi.fn();

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  describe("内置插件组件渲染", () => {
    it("应该正确渲染 machine-id-tool 插件", () => {
      const { container } = renderComponent(
        <PluginUIRenderer
          pluginId="machine-id-tool"
          onNavigate={mockNavigate}
        />,
      );

      // 验证 MachineIdTool 组件被渲染
      const machineIdTool = container.querySelector(
        '[data-testid="machine-id-tool"]',
      );
      expect(machineIdTool).not.toBeNull();
      expect(machineIdTool?.textContent).toBe("MachineIdTool Mock");
    });
  });

  describe("未知插件处理", () => {
    it("应该为未知插件显示 '插件未找到' 提示", () => {
      const { container } = renderComponent(
        <PluginUIRenderer
          pluginId="unknown-plugin"
          onNavigate={mockNavigate}
        />,
      );

      // 验证显示插件未找到提示
      expect(container.textContent).toContain("插件未找到");
      expect(container.textContent).toContain("unknown-plugin");
      expect(container.textContent).toContain("未安装或不存在");
    });

    it("应该为空字符串 pluginId 显示 '插件未找到' 提示", () => {
      const { container } = renderComponent(
        <PluginUIRenderer pluginId="" onNavigate={mockNavigate} />,
      );

      // 验证显示插件未找到提示
      expect(container.textContent).toContain("插件未找到");
    });

    it("应该为随机 pluginId 显示 '插件未找到' 提示", () => {
      const randomPluginId = `random-plugin-${Date.now()}`;
      const { container } = renderComponent(
        <PluginUIRenderer
          pluginId={randomPluginId}
          onNavigate={mockNavigate}
        />,
      );

      // 验证显示插件未找到提示，并包含插件 ID
      expect(container.textContent).toContain("插件未找到");
      expect(container.textContent).toContain(randomPluginId);
    });
  });

  describe("插件 ID 大小写敏感性", () => {
    it("应该区分大小写 - 'Machine-Id-Tool' 应该显示未找到", () => {
      const { container } = renderComponent(
        <PluginUIRenderer
          pluginId="Machine-Id-Tool"
          onNavigate={mockNavigate}
        />,
      );

      // 验证大小写不匹配时显示未找到
      expect(container.textContent).toContain("插件未找到");
    });

    it("应该区分大小写 - 'MACHINE-ID-TOOL' 应该显示未找到", () => {
      const { container } = renderComponent(
        <PluginUIRenderer
          pluginId="MACHINE-ID-TOOL"
          onNavigate={mockNavigate}
        />,
      );

      // 验证大小写不匹配时显示未找到
      expect(container.textContent).toContain("插件未找到");
    });
  });
});

/**
 * 简单的渲染辅助函数
 * 使用 jsdom 环境渲染 React 组件
 */
function renderComponent(element: React.ReactElement) {
  const container = document.createElement("div");
  document.body.appendChild(container);

  // 使用 React 18 的 createRoot API
  const root = createRoot(container);

  // 临时禁用 console.error 来抑制 act 警告
  const originalError = console.error;
  console.error = (...args: unknown[]) => {
    if (typeof args[0] === "string" && args[0].includes("act(...)")) {
      return;
    }
    originalError.apply(console, args);
  };

  act(() => {
    root.render(element);
  });

  // 恢复 console.error
  console.error = originalError;

  return {
    container,
    unmount: () => {
      act(() => {
        root.unmount();
      });
      container.remove();
    },
  };
}
