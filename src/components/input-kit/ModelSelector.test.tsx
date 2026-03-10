import React from "react";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const {
  mockUseConfiguredProviders,
  mockUseProviderModels,
  mockFilterModelsByTheme,
} = vi.hoisted(() => ({
  mockUseConfiguredProviders: vi.fn(),
  mockUseProviderModels: vi.fn(),
  mockFilterModelsByTheme: vi.fn(),
}));

vi.mock("@/hooks/useConfiguredProviders", () => ({
  useConfiguredProviders: () => mockUseConfiguredProviders(),
}));

vi.mock("@/hooks/useProviderModels", () => ({
  useProviderModels: (...args: unknown[]) => mockUseProviderModels(...args),
}));

vi.mock("@/components/agent/chat/utils/modelThemePolicy", () => ({
  filterModelsByTheme: (...args: unknown[]) => mockFilterModelsByTheme(...args),
}));

import { ModelSelector } from "./ModelSelector";

interface MountedRoot {
  root: Root;
  container: HTMLDivElement;
}

const mountedRoots: MountedRoot[] = [];

function renderModelSelector(
  props: Partial<React.ComponentProps<typeof ModelSelector>> = {},
) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const root = createRoot(container);

  const mergedProps: React.ComponentProps<typeof ModelSelector> = {
    providerType: "custom-codex",
    setProviderType: vi.fn(),
    model: "gpt-5.3-codex",
    setModel: vi.fn(),
    activeTheme: "general",
    ...props,
  };

  act(() => {
    root.render(<ModelSelector {...mergedProps} />);
  });

  mountedRoots.push({ root, container });
  return { container, props: mergedProps };
}

beforeEach(() => {
  (
    globalThis as typeof globalThis & {
      IS_REACT_ACT_ENVIRONMENT?: boolean;
    }
  ).IS_REACT_ACT_ENVIRONMENT = true;

  vi.clearAllMocks();

  mockUseConfiguredProviders.mockReturnValue({
    providers: [
      {
        key: "custom-codex",
        label: "Codex Custom",
        registryId: "custom-codex",
        fallbackRegistryId: "codex",
        type: "codex",
      },
    ],
    loading: false,
  });

  mockUseProviderModels.mockReturnValue({
    modelIds: ["gpt-5.3-codex", "gpt-5.2-codex"],
    models: [
      { id: "gpt-5.3-codex" },
      { id: "gpt-5.2-codex" },
    ],
    loading: false,
    error: null,
  });

  mockFilterModelsByTheme.mockImplementation((_theme, models) => ({
    models,
    usedFallback: false,
    filteredOutCount: 0,
    policyName: "none",
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
});

describe("ModelSelector", () => {
  it("应在 codex 不兼容模型被选中时自动回退到兼容模型", () => {
    const setModel = vi.fn();

    renderModelSelector({
      model: "gpt-5.3-codex",
      setModel,
    });

    expect(setModel).toHaveBeenCalledWith("gpt-5.2-codex");
  });
});
