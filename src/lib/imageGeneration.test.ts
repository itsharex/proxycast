import { describe, expect, it } from "vitest";
import {
  findImageProviderById,
  findImageProviderForSelection,
  getImageModelsForProvider,
  isImageProvider,
  pickImageModelBySelection,
  type ImageProviderCandidate,
} from "./imageGeneration";

interface MockProvider extends ImageProviderCandidate {
  name: string;
}

const providers: MockProvider[] = [
  { id: "new-api", type: "openai", name: "OpenAI 兼容" },
  { id: "doubao-image", type: "openai", name: "即梦" },
  { id: "kling", type: "openai", name: "可灵" },
];

describe("imageGeneration", () => {
  it("应识别图片 Provider", () => {
    expect(isImageProvider("new-api", "openai")).toBe(true);
    expect(isImageProvider("tts-only", "audio")).toBe(false);
  });

  it("应按项目配置优先匹配指定 Provider", () => {
    expect(findImageProviderById(providers, "doubao-image")?.name).toBe("即梦");
    expect(findImageProviderById(providers, "missing-provider")).toBeNull();
  });

  it("应按预设模型偏好自动选择 Provider", () => {
    expect(findImageProviderForSelection(providers, "basic")?.id).toBe("new-api");
    expect(findImageProviderForSelection(providers, "jimeng")?.id).toBe(
      "doubao-image",
    );
    expect(findImageProviderForSelection(providers, "kling")?.id).toBe("kling");
  });

  it("应按预设模型偏好自动选择模型", () => {
    expect(pickImageModelBySelection([], "basic")).toBe("gpt-image-1");
    expect(
      pickImageModelBySelection(["flux-pro", "gpt-image-1"], "basic"),
    ).toBe("gpt-image-1");
    expect(pickImageModelBySelection([], "jimeng")).toBe("seedream-3.0");
  });

  it("应解析 Provider 可用模型列表", () => {
    expect(getImageModelsForProvider("new-api", "openai")[0]?.id).toBe("dall-e-3");
    expect(
      getImageModelsForProvider("custom-provider", "openai", ["gpt-image-1"])[0]
        ?.id,
    ).toBe("gpt-image-1");
  });
});
