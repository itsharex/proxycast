import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import {
  emitImageWorkbenchFocus,
  emitImageWorkbenchRequest,
} from "@/lib/imageWorkbenchEvents";
import type { GeneratedOutputItem } from "./workbenchRightRailGeneratedOutputs";
import type {
  CoverCountType,
  CoverPlatformType,
  ImageModelType,
  ImageSizeType,
  SearchResourceType,
} from "./workbenchRightRailCreationConfig";
import {
  COVER_PLATFORM_OPTIONS,
  IMAGE_MODEL_OPTIONS,
  mapCoverPlatformToAspectRatio,
  mapImageSizeTypeToAspectRatio,
  SEARCH_RESOURCE_OPTIONS,
} from "./workbenchRightRailCreationConfig";
import {
  getOptionLabel,
  type WebImageSearchResponseForRail,
} from "./workbenchRightRailCapabilityShared";

interface UseWorkbenchRightRailImageTasksParams {
  projectId?: string | null;
  contentId?: string | null;
  appendGeneratedOutput: (item: GeneratedOutputItem) => void;
  handleSubmitPrompt: (prompt: string) => Promise<boolean>;
}

export function useWorkbenchRightRailImageTasks({
  projectId,
  contentId,
  appendGeneratedOutput,
  handleSubmitPrompt,
}: UseWorkbenchRightRailImageTasksParams) {
  const [searchResourceType, setSearchResourceType] =
    useState<SearchResourceType>("image");
  const [searchQuery, setSearchQuery] = useState("");
  const [searchMaterialSubmitting, setSearchMaterialSubmitting] =
    useState(false);
  const [searchMaterialResultSummary, setSearchMaterialResultSummary] =
    useState("");
  const [titleRequirement, setTitleRequirement] = useState("");
  const [imageModel, setImageModel] = useState<ImageModelType>("basic");
  const [imageSize, setImageSize] = useState<ImageSizeType>("16-9");
  const [imagePrompt, setImagePrompt] = useState("");
  const [coverPlatform, setCoverPlatform] =
    useState<CoverPlatformType>("bilibili");
  const [coverCount, setCoverCount] = useState<CoverCountType>("1");
  const [coverDescription, setCoverDescription] = useState("");
  const [imageSubmitting, setImageSubmitting] = useState(false);
  const [coverSubmitting, setCoverSubmitting] = useState(false);

  const submitImageWorkbenchRequest = async (params: {
    prompt: string;
    target: "generate" | "cover";
    aspectRatio: string;
    count?: number;
    modelPreset: ImageModelType;
    outputTitle: string;
    outputDetail: string;
  }) => {
    const normalizedPrompt = params.prompt.trim();
    if (!normalizedPrompt) {
      return false;
    }

    if (contentId) {
      emitImageWorkbenchRequest({
        source: "workspace-right-rail",
        projectId: projectId ?? null,
        contentId,
        prompt: normalizedPrompt,
        target: params.target,
        aspectRatio: params.aspectRatio,
        count: params.count,
        modelPreset: params.modelPreset,
      });
      appendGeneratedOutput({
        id: `image-workbench-${Date.now()}`,
        title: params.outputTitle,
        detail: params.outputDetail,
        actionLabel: "查看画布",
        onAction: () => {
          emitImageWorkbenchFocus({
            source: "workspace-right-rail",
            projectId: projectId ?? null,
            contentId,
          });
        },
      });
      toast.success("已提交到图片工作台");
      return true;
    }

    const command = [
      `@配图 生成 ${normalizedPrompt}`,
      params.aspectRatio ? `，${params.aspectRatio}` : "",
      params.count && params.count > 1 ? `，出 ${params.count} 张` : "",
    ].join("");
    const submitted = await handleSubmitPrompt(command);
    if (submitted) {
      appendGeneratedOutput({
        id: `image-workbench-${Date.now()}`,
        title: params.outputTitle,
        detail: `${params.outputDetail} 已转为工作区创建请求。`,
      });
    }
    return submitted;
  };

  const handleSubmitSearchMaterial = async () => {
    const normalizedQuery = searchQuery.trim();
    if (!normalizedQuery) {
      return;
    }

    if (searchResourceType !== "image") {
      const resourceTypeLabel = getOptionLabel(
        SEARCH_RESOURCE_OPTIONS,
        searchResourceType,
      );
      const submitted =
        await handleSubmitPrompt(`请帮我检索${resourceTypeLabel}素材，关键词：${normalizedQuery}。
请输出可直接使用的素材建议与来源。`);
      if (submitted) {
        appendGeneratedOutput({
          id: `search-${Date.now()}`,
          title: `${resourceTypeLabel}检索任务已提交`,
          detail: `关键词：${normalizedQuery}`,
        });
      }
      return;
    }

    setSearchMaterialSubmitting(true);
    try {
      const response = await invoke<WebImageSearchResponseForRail>(
        "search_web_images",
        {
          req: {
            query: normalizedQuery,
            page: 1,
            perPage: 10,
          },
        },
      );
      const count = response.hits?.length ?? 0;
      const total = response.total ?? count;
      const provider = response.provider || "web";
      const previewName = response.hits?.[0]?.name?.trim() || "无标题素材";
      const summary = `已检索到 ${total} 条结果（当前返回 ${count} 条，来源：${provider}，示例：${previewName}）。`;
      setSearchMaterialResultSummary(summary);
      appendGeneratedOutput({
        id: `search-${Date.now()}`,
        title: "素材检索完成",
        detail: summary,
      });
      toast.success("素材搜索完成");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setSearchMaterialResultSummary("");
      toast.error(`素材搜索失败：${message}`);
    } finally {
      setSearchMaterialSubmitting(false);
    }
  };

  const handleSubmitTitleTask = async () => {
    await handleSubmitPrompt(`请根据以下要求生成 10 个标题候选，标题需风格多样且避免重复：
${titleRequirement.trim()}`);
  };

  const handleSubmitImageTask = async () => {
    const normalizedPrompt = imagePrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    setImageSubmitting(true);
    try {
      const aspectRatio = mapImageSizeTypeToAspectRatio(imageSize);
      const modelLabel = getOptionLabel(IMAGE_MODEL_OPTIONS, imageModel);
      const submitted = await submitImageWorkbenchRequest({
        prompt: normalizedPrompt,
        target: "generate",
        aspectRatio,
        count: 1,
        modelPreset: imageModel,
        outputTitle: "图片任务已提交",
        outputDetail: `${modelLabel} · ${aspectRatio} · ${normalizedPrompt}`,
      });
      if (submitted) {
        setImagePrompt("");
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`图片任务提交失败：${message}`);
    } finally {
      setImageSubmitting(false);
    }
  };

  const handleSubmitCoverTask = async () => {
    const normalizedPrompt = coverDescription.trim();
    if (!normalizedPrompt) {
      return;
    }

    setCoverSubmitting(true);
    try {
      const imageCount = Number.parseInt(coverCount, 10);
      const platformLabel = getOptionLabel(
        COVER_PLATFORM_OPTIONS,
        coverPlatform,
      );
      const aspectRatio = mapCoverPlatformToAspectRatio(coverPlatform);
      const coverPrompt = `请生成${platformLabel}平台封面图。要求：${normalizedPrompt}`;
      const modelLabel = getOptionLabel(IMAGE_MODEL_OPTIONS, imageModel);
      const submitted = await submitImageWorkbenchRequest({
        prompt: coverPrompt,
        target: "cover",
        aspectRatio,
        count: Number.isFinite(imageCount) ? imageCount : 1,
        modelPreset: imageModel,
        outputTitle: "封面任务已提交",
        outputDetail: `${platformLabel} · ${modelLabel} · ${aspectRatio} · ${
          Number.isFinite(imageCount) ? imageCount : 1
        } 张`,
      });
      if (submitted) {
        setCoverDescription("");
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`封面任务提交失败：${message}`);
    } finally {
      setCoverSubmitting(false);
    }
  };

  const closeSearchMaterialPanel = () => {
    setSearchMaterialResultSummary("");
  };

  return {
    closeSearchMaterialPanel,
    coverCount,
    coverDescription,
    coverPlatform,
    coverSubmitting,
    handleSubmitCoverTask,
    handleSubmitImageTask,
    handleSubmitSearchMaterial,
    handleSubmitTitleTask,
    imageModel,
    imagePrompt,
    imageSize,
    imageSubmitting,
    searchMaterialResultSummary,
    searchMaterialSubmitting,
    searchQuery,
    searchResourceType,
    setCoverCount,
    setCoverDescription,
    setCoverPlatform,
    setImageModel,
    setImagePrompt,
    setImageSize,
    setSearchQuery,
    setSearchResourceType,
    setTitleRequirement,
    titleRequirement,
  };
}
