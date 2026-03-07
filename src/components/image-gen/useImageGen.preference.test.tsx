import { act } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanupMountedRoots,
  flushEffects,
  renderIntoDom,
  setReactActEnvironment,
  type MountedRoot,
} from "./test-utils";

vi.mock("@/hooks/useApiKeyProvider", () => ({
  useApiKeyProvider: () => ({
    providers: [
      {
        id: "zhipuai",
        type: "zhipuai",
        name: "智谱AI",
        enabled: true,
        api_key_count: 1,
        api_host: "https://api.zhipu.test",
      },
      {
        id: "fal",
        type: "fal",
        name: "Fal",
        enabled: true,
        api_key_count: 1,
        api_host: "https://fal.run",
      },
    ],
    loading: false,
  }),
}));

import { useImageGen } from "./useImageGen";

interface HookHarness {
  getValue: () => ReturnType<typeof useImageGen>;
}

const mountedRoots: MountedRoot[] = [];

function mountHook(preferredProviderId?: string): HookHarness {
  let hookValue: ReturnType<typeof useImageGen> | null = null;

  function TestComponent() {
    hookValue = useImageGen({ preferredProviderId });
    return null;
  }

  renderIntoDom(<TestComponent />, mountedRoots);

  return {
    getValue: () => {
      if (!hookValue) {
        throw new Error("hook 尚未初始化");
      }
      return hookValue;
    },
  };
}

function mountHookWithOptions(options: {
  preferredProviderId?: string;
  preferredModelId?: string;
}): HookHarness {
  let hookValue: ReturnType<typeof useImageGen> | null = null;

  function TestComponent() {
    hookValue = useImageGen(options);
    return null;
  }

  renderIntoDom(<TestComponent />, mountedRoots);

  return {
    getValue: () => {
      if (!hookValue) {
        throw new Error("hook 尚未初始化");
      }
      return hookValue;
    },
  };
}

async function waitForReady(harness: HookHarness, timeout = 40): Promise<void> {
  for (let i = 0; i < timeout; i += 1) {
    const value = harness.getValue();
    if (value.selectedProvider && value.selectedModelId) {
      return;
    }
    await flushEffects();
  }
  throw new Error("useImageGen 未在预期时间内就绪");
}

beforeEach(() => {
  setReactActEnvironment();
});

afterEach(() => {
  cleanupMountedRoots(mountedRoots);
  vi.restoreAllMocks();
});

describe("useImageGen 项目偏好", () => {
  it("应优先选择项目指定的图片 Provider 和模型", async () => {
    const harness = mountHook("fal");
    await act(async () => {
      await waitForReady(harness);
    });

    expect(harness.getValue().selectedProvider?.id).toBe("fal");
    expect(harness.getValue().selectedModelId).toBe("fal-ai/nano-banana-pro");
  });

  it("应优先选择项目指定的图片模型", async () => {
    const harness = mountHookWithOptions({
      preferredProviderId: "fal",
      preferredModelId: "fal-ai/flux-kontext/dev",
    });
    await act(async () => {
      await waitForReady(harness);
    });

    expect(harness.getValue().selectedProvider?.id).toBe("fal");
    expect(harness.getValue().selectedModelId).toBe("fal-ai/flux-kontext/dev");
  });
});
