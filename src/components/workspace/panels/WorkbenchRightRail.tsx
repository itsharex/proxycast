import { useEffect, useRef, useState } from "react";
import {
  AudioLines,
  Bot,
  Check,
  ChevronDown,
  Clapperboard,
  FileSearch,
  Film,
  Image,
  LayoutTemplate,
  Mic,
  Music4,
  PanelRightClose,
  PanelRightOpen,
  Search,
  Sparkles,
  Type,
  User,
  Users,
  Video,
  WandSparkles,
  type LucideIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { Dialog, DialogContent } from "@/components/ui/dialog";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { toast } from "sonner";
import { apiKeyProviderApi } from "@/lib/api/apiKeyProvider";
import { videoGenerationApi } from "@/lib/api/videoGeneration";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types/project";
import {
  findImageProviderById,
  findImageProviderForSelection,
  getImageModelIdsForProvider,
  isImageProvider,
  pickImageModelBySelection,
} from "@/lib/imageGeneration";
import {
  findMediaProviderById,
  findTtsProviderForSelection,
  findVideoProviderForSelection,
  getTtsModelsForProvider,
  getVideoModelsForProvider,
  isTtsProvider,
  isVideoProvider,
  pickTtsModel,
  pickVideoModelByVersion,
  resolveMediaGenerationPreference,
} from "@/lib/mediaGeneration";
import { useGlobalMediaGenerationDefaults } from "@/hooks/useGlobalMediaGenerationDefaults";
import { ContentReviewPanel } from "@/components/content-creator/canvas/document/ContentReviewPanel";
import { useWorkbenchStore } from "@/stores/useWorkbenchStore";
import { ThemeWorkbenchSkillsPanel } from "@/components/agent/chat/components/ThemeWorkbenchSkillsPanel";

export interface WorkbenchRightRailProps {
  shouldRender: boolean;
  isCreateWorkspaceView: boolean;
  projectId?: string | null;
  onBackToCreateView: () => void;
  onCreateContentFromPrompt?: (prompt: string) => Promise<void> | void;
}

interface CapabilityItem {
  key: string;
  label: string;
  icon: LucideIcon;
  tone: "violet" | "blue" | "pink";
}

interface CapabilitySection {
  key: string;
  title: string;
  tone: "violet" | "blue" | "pink";
  items: CapabilityItem[];
}

interface GeneratedOutputItem {
  id: string;
  title: string;
  detail: string;
  assetType?: "image" | "audio";
  assetUrl?: string;
}

type SearchResourceType = "image" | "audio" | "bgm";
type ImageModelType = "basic" | "jimeng" | "kling";
type ImageSizeType = "16-9" | "9-16" | "1-1";
type CoverPlatformType = "bilibili" | "xiaohongshu" | "douyin";
type CoverCountType = "1" | "2" | "3";
type VideoAssetModelType = "keling" | "jimeng" | "wan-2-5";
type VideoAssetVersionType = "v2-1-master" | "v2" | "v1-6";
type VideoAssetRatioType = "16-9" | "9-16" | "1-1";
type VideoAssetDurationType = "5s";
type VoiceoverSpeedType = "0.8x" | "1.0x" | "1.2x";
type VoiceoverToneTabType = "mine" | "library";
type VoiceoverToneId =
  | "gaolengyujie"
  | "aojiaobazong"
  | "shuangkuaisisi"
  | "wennuanahu"
  | "shaonianzixin"
  | "yuanboxiaoshu"
  | "yangguangqingnian"
  | "wanwanxiaohe";
type BgmDurationType = "30s";
type SfxDurationType = "10s";
type PodcastModeType = "deep" | "quick" | "debate";
type PodcastSpeakerModeType = "dual" | "single";

interface ImageProviderOption {
  id: string;
  type: string;
  apiHost: string;
  customModels: string[];
}

interface TtsProviderOption {
  id: string;
  type: string;
  apiHost: string;
  customModels: string[];
}

interface WebImageSearchResponseForRail {
  total: number;
  provider: string;
  hits: Array<{
    id: string;
    name: string;
    content_url?: string;
    contentUrl?: string;
  }>;
}

const SEARCH_RESOURCE_OPTIONS: Array<{
  value: SearchResourceType;
  label: string;
}> = [
  { value: "audio", label: "音效" },
  { value: "bgm", label: "背景音乐" },
  { value: "image", label: "图片" },
];

const IMAGE_MODEL_OPTIONS: Array<{
  value: ImageModelType;
  label: string;
  disabled?: boolean;
}> = [
  { value: "basic", label: "基础模型" },
  { value: "jimeng", label: "即梦" },
  { value: "kling", label: "可灵", disabled: true },
];

const IMAGE_SIZE_OPTIONS: Array<{
  value: ImageSizeType;
  label: string;
}> = [
  { value: "16-9", label: "16:9 横图" },
  { value: "9-16", label: "9:16 竖图" },
  { value: "1-1", label: "1:1 方图" },
];

const COVER_PLATFORM_OPTIONS: Array<{
  value: CoverPlatformType;
  label: string;
}> = [
  { value: "xiaohongshu", label: "小红书" },
  { value: "douyin", label: "抖音" },
  { value: "bilibili", label: "B站" },
];

const COVER_COUNT_OPTIONS: Array<{
  value: CoverCountType;
  label: string;
}> = [
  { value: "3", label: "3 张" },
  { value: "2", label: "2 张" },
  { value: "1", label: "1 张" },
];

const VIDEO_ASSET_MODEL_OPTIONS: Array<{
  value: VideoAssetModelType;
  label: string;
  disabled?: boolean;
}> = [
  { value: "keling", label: "可灵" },
  { value: "jimeng", label: "即梦" },
  { value: "wan-2-5", label: "WAN 2.5", disabled: true },
];

const VIDEO_ASSET_VERSION_OPTIONS: Array<{
  value: VideoAssetVersionType;
  label: string;
}> = [
  { value: "v2", label: "V2" },
  { value: "v1-6", label: "V1-6" },
  { value: "v2-1-master", label: "V2.1 Master" },
];

const VIDEO_ASSET_RATIO_OPTIONS: Array<{
  value: VideoAssetRatioType;
  label: string;
}> = [
  { value: "1-1", label: "1:1" },
  { value: "9-16", label: "9:16" },
  { value: "16-9", label: "16:9" },
];

const VIDEO_ASSET_DURATION_OPTIONS: Array<{
  value: VideoAssetDurationType;
  label: string;
}> = [{ value: "5s", label: "5s" }];

const VOICEOVER_SPEED_OPTIONS: Array<{
  value: VoiceoverSpeedType;
  label: string;
}> = [
  { value: "0.8x", label: "0.8x" },
  { value: "1.0x", label: "1.0x" },
  { value: "1.2x", label: "1.2x" },
];

const VOICEOVER_TONE_OPTIONS: Array<{
  id: VoiceoverToneId;
  label: string;
  gender: "男声" | "女声";
}> = [
  { id: "gaolengyujie", label: "高冷御姐", gender: "女声" },
  { id: "aojiaobazong", label: "傲娇霸总", gender: "男声" },
  { id: "shuangkuaisisi", label: "爽快思思", gender: "女声" },
  { id: "wennuanahu", label: "温暖阿虎", gender: "男声" },
  { id: "shaonianzixin", label: "少年梓辛", gender: "男声" },
  { id: "yuanboxiaoshu", label: "渊博小叔", gender: "男声" },
  { id: "yangguangqingnian", label: "阳光青年", gender: "男声" },
  { id: "wanwanxiaohe", label: "湾湾小何", gender: "女声" },
];

const BGM_DURATION_OPTIONS: Array<{
  value: BgmDurationType;
  label: string;
}> = [{ value: "30s", label: "30s" }];

const SFX_DURATION_OPTIONS: Array<{
  value: SfxDurationType;
  label: string;
}> = [{ value: "10s", label: "10s" }];

const PODCAST_MODE_OPTIONS: Array<{
  value: PodcastModeType;
  label: string;
}> = [
  { value: "deep", label: "深度模式" },
  { value: "quick", label: "快速模式" },
  { value: "debate", label: "辩论模式" },
];

const PODCAST_QUICK_IMPORT_PROMPT = `Agent 炒作何时停？2026 年，请让智能体走下神坛
2026年，AI Agent（智能体）终于从 PPT 里的“万能灵药”变成了企业报表里的“成本项”。当 Computer Use 成为标配，当多智能体协作（Multi-Agent Systems）开始编织数字流水线，那个曾经被吹得天花乱坠的泡沫，终于开始加速破裂。
一、 从“会聊”到“会办”，红利期已过
如果说 2024 年我们还在为 Agent 能写一段代码而欢呼，那么 2026 年的职场早已习惯了嵌入式 AI 的存在。无论是 Salesforce 还是 Office 365，原生智能体已经接管了繁琐的 ERP 和 CRM 操作。这种“去工具化”趋势意味着，单纯靠底层模型能力包装的 Agent 已经失去了溢价能力。
二、 40% 的项目失败率：CFO 的冷酷校准
今年是 AI 行业的“Show me the money”之年。根据行业调研，约 40% 的 Agent 项目因无法量化 ROI（投资回报率）而宣告失败。CFO 们不再听信“改变生产力”的宏大叙事，他们只关心 Token 消耗的成本黑洞与实际业务产出是否对等。那些只有 Demo、没有垂直场景深耕的“套壳”公司，正迎来最残酷的倒闭潮。
三、 落地之痛：数据孤岛与信任危机
尽管技术在飞跃，但 Agent 依然被困在企业内部的数据孤岛中。跨系统的权限摩擦、自主决策带来的安全隐患，以及由于逻辑复杂导致的执行幻觉，让很多企业在临门一脚时选择了保守。Agent 想要真正“接管”工作，需要的不仅是更聪明的 LLM，更是底层业务流程的彻底重构。
四、 结语：泡沫退去，方见真章
Agent 的炒作不会消失，但会“降温”。当市场不再盲目追求通用智能，转而关注那些能扎根在金融、医疗、供应链等垂直领域默默干活的“数字员工”时，AI 才算真正走入了深水区。Agent 炒作何时停？当它不再是新闻，而变成像水电一样的基础设施时，它才真正成功了。`;

function getOptionLabel<TValue extends string>(
  options: Array<{ value: TValue; label: string }>,
  value: TValue,
): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

interface VideoProviderOption {
  id: string;
  customModels: string[];
}

function parseVideoDuration(duration: VideoAssetDurationType): number {
  const value = Number.parseInt(duration.replace("s", ""), 10);
  return Number.isFinite(value) ? value : 5;
}

function mapImageSizeTypeToResolution(size: ImageSizeType): string {
  if (size === "16-9") {
    return "1792x1024";
  }
  if (size === "9-16") {
    return "1024x1792";
  }
  return "1024x1024";
}

function mapCoverPlatformToResolution(platform: CoverPlatformType): string {
  if (platform === "bilibili") {
    return "1792x1024";
  }
  if (platform === "douyin") {
    return "1024x1792";
  }
  return "1024x1792";
}

function buildProviderEndpoint(apiHost: string, endpointPath: string): string {
  const trimmedHost = (apiHost || "").trim().replace(/\/+$/, "");
  const normalizedPath = endpointPath.startsWith("/")
    ? endpointPath
    : `/${endpointPath}`;
  return `${trimmedHost}${normalizedPath}`;
}

function mapToneToTtsVoice(toneId: VoiceoverToneId): string {
  const toneMap: Record<VoiceoverToneId, string> = {
    gaolengyujie: "alloy",
    aojiaobazong: "onyx",
    shuangkuaisisi: "nova",
    wennuanahu: "echo",
    shaonianzixin: "fable",
    yuanboxiaoshu: "onyx",
    yangguangqingnian: "echo",
    wanwanxiaohe: "shimmer",
  };
  return toneMap[toneId] ?? "alloy";
}

function parseVoiceSpeed(speed: VoiceoverSpeedType): number {
  const value = Number.parseFloat(speed.replace("x", ""));
  return Number.isFinite(value) ? value : 1;
}

function parseSimpleDuration(duration: string, fallbackValue: number): number {
  const value = Number.parseInt(duration.replace(/[^\d]/g, ""), 10);
  return Number.isFinite(value) ? value : fallbackValue;
}

function convertBlobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    if (typeof FileReader === "undefined") {
      reject(new Error("当前环境不支持 FileReader"));
      return;
    }
    const reader = new FileReader();
    reader.onerror = () => {
      reject(new Error("音频读取失败"));
    };
    reader.onload = () => {
      const result = typeof reader.result === "string" ? reader.result : "";
      if (!result) {
        reject(new Error("音频读取结果为空"));
        return;
      }
      resolve(result);
    };
    reader.readAsDataURL(blob);
  });
}

function revokeObjectUrlIfNeeded(url: string): void {
  if (
    url.startsWith("blob:") &&
    typeof URL !== "undefined" &&
    typeof URL.revokeObjectURL === "function"
  ) {
    URL.revokeObjectURL(url);
  }
}

function tryParseJsonText(text: string): Record<string, unknown> | null {
  try {
    const parsed = JSON.parse(text) as unknown;
    if (parsed && typeof parsed === "object") {
      return parsed as Record<string, unknown>;
    }
    return null;
  } catch {
    return null;
  }
}

function extractImageUrlsFromResponse(
  payload: Record<string, unknown> | null,
): string[] {
  const data = payload?.data;
  if (!Array.isArray(data)) {
    return [];
  }

  return data
    .map((item) => {
      if (!item || typeof item !== "object") {
        return "";
      }
      const record = item as Record<string, unknown>;
      const url = typeof record.url === "string" ? record.url.trim() : "";
      if (url) {
        return url;
      }
      const b64 = typeof record.b64_json === "string" ? record.b64_json : "";
      if (!b64) {
        return "";
      }
      return `data:image/png;base64,${b64}`;
    })
    .filter((url) => url.length > 0);
}

function extractAudioUrlFromResponse(
  payload: Record<string, unknown> | null,
): string {
  if (!payload) {
    return "";
  }

  const directUrl = typeof payload.url === "string" ? payload.url.trim() : "";
  if (directUrl) {
    return directUrl;
  }

  const directB64 =
    typeof payload.b64_json === "string" ? payload.b64_json : "";
  if (directB64) {
    return `data:audio/mpeg;base64,${directB64}`;
  }

  const data = payload.data;
  if (!Array.isArray(data)) {
    return "";
  }

  for (const item of data) {
    if (!item || typeof item !== "object") {
      continue;
    }
    const record = item as Record<string, unknown>;
    const itemUrl = typeof record.url === "string" ? record.url.trim() : "";
    if (itemUrl) {
      return itemUrl;
    }
    const itemB64 = typeof record.b64_json === "string" ? record.b64_json : "";
    if (itemB64) {
      return `data:audio/mpeg;base64,${itemB64}`;
    }
  }

  return "";
}

const CAPABILITY_SECTIONS: CapabilitySection[] = [
  {
    key: "text-search",
    title: "文字多搜索",
    tone: "violet",
    items: [
      {
        key: "search-material",
        label: "搜索素材",
        icon: Search,
        tone: "violet",
      },
      { key: "generate-title", label: "生成标题", icon: Type, tone: "violet" },
    ],
  },
  {
    key: "visual",
    title: "视觉生成",
    tone: "blue",
    items: [
      { key: "generate-image", label: "生成图片", icon: Image, tone: "blue" },
      {
        key: "generate-cover",
        label: "生成封面",
        icon: LayoutTemplate,
        tone: "blue",
      },
      {
        key: "generate-storyboard",
        label: "生成分镜",
        icon: Film,
        tone: "blue",
      },
      {
        key: "generate-video-assets",
        label: "生成视频素材",
        icon: Clapperboard,
        tone: "blue",
      },
      {
        key: "generate-ai-video",
        label: "生成视频(非AI画面)",
        icon: Video,
        tone: "blue",
      },
    ],
  },
  {
    key: "audio",
    title: "音频生成",
    tone: "pink",
    items: [
      { key: "generate-voiceover", label: "生成配音", icon: Mic, tone: "pink" },
      { key: "generate-bgm", label: "生成BGM", icon: Music4, tone: "pink" },
      {
        key: "generate-sfx",
        label: "生成音效",
        icon: AudioLines,
        tone: "pink",
      },
      {
        key: "generate-podcast",
        label: "生成播客",
        icon: WandSparkles,
        tone: "pink",
      },
    ],
  },
];

const SECTION_TONE_CLASS: Record<CapabilitySection["tone"], string> = {
  violet: "text-violet-500",
  blue: "text-blue-500",
  pink: "text-pink-500",
};

const CARD_TONE_CLASS: Record<CapabilityItem["tone"], string> = {
  violet:
    "border-violet-100 bg-violet-50/80 text-violet-500 hover:border-violet-200 hover:bg-violet-50",
  blue: "border-blue-100 bg-blue-50/80 text-blue-500 hover:border-blue-200 hover:bg-blue-50",
  pink: "border-pink-100 bg-pink-50/80 text-pink-500 hover:border-pink-200 hover:bg-pink-50",
};

function SearchMaterialPanel({
  resourceType,
  searchQuery,
  isSubmitting,
  resultSummary,
  onResourceTypeChange,
  onSearchQueryChange,
  onSubmit,
  onCancel,
}: {
  resourceType: SearchResourceType;
  searchQuery: string;
  isSubmitting?: boolean;
  resultSummary?: string;
  onResourceTypeChange: (value: SearchResourceType) => void;
  onSearchQueryChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-violet-100 bg-white p-4 shadow-sm"
      data-testid="workbench-search-material-panel"
    >
      <div className="rounded-2xl border border-violet-100 bg-violet-50/70 px-4 py-3 text-violet-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Search className="h-3.5 w-3.5" />
          <span>搜索素材</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">资源类型</div>
          <Select
            value={resourceType}
            onValueChange={(value) =>
              onResourceTypeChange(value as SearchResourceType)
            }
          >
            <SelectTrigger className="h-9 w-[88px] rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent side="bottom" className="min-w-[120px]">
              {SEARCH_RESOURCE_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">搜索词</div>
          <Textarea
            value={searchQuery}
            onChange={(event) => onSearchQueryChange(event.target.value)}
            placeholder="请输入搜索词"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-violet-200"
          />
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!searchQuery.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "搜索中..." : "提交"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>

        {resultSummary ? (
          <div className="rounded-xl border border-violet-100 bg-violet-50/50 px-3 py-2 text-[12px] text-violet-700">
            {resultSummary}
          </div>
        ) : null}
      </div>
    </div>
  );
}

function GenerateTitlePanel({
  requirement,
  onRequirementChange,
  onSubmit,
  onCancel,
}: {
  requirement: string;
  onRequirementChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-violet-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-title-panel"
    >
      <div className="rounded-2xl border border-violet-100 bg-violet-50/70 px-4 py-3 text-violet-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Type className="h-3.5 w-3.5" />
          <span>生成标题</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">要求</div>
          <Textarea
            value={requirement}
            onChange={(event) => onRequirementChange(event.target.value)}
            placeholder="请输入要求"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-violet-200"
          />
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!requirement.trim()}
            onClick={onSubmit}
          >
            一键生成
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function GenerateImagePanel({
  model,
  size,
  prompt,
  isSubmitting,
  onModelChange,
  onSizeChange,
  onPromptChange,
  onSubmit,
  onCancel,
}: {
  model: ImageModelType;
  size: ImageSizeType;
  prompt: string;
  isSubmitting?: boolean;
  onModelChange: (value: ImageModelType) => void;
  onSizeChange: (value: ImageSizeType) => void;
  onPromptChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-blue-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-image-panel"
    >
      <div className="rounded-2xl border border-blue-100 bg-blue-50/70 px-4 py-3 text-blue-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Image className="h-3.5 w-3.5" />
          <span>生成图片</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">模型</div>
            <Select
              value={model}
              onValueChange={(value) => onModelChange(value as ImageModelType)}
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {IMAGE_MODEL_OPTIONS.filter((option) => !option.disabled).map(
                  (option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ),
                )}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">尺寸</div>
            <Select
              value={size}
              onValueChange={(value) => onSizeChange(value as ImageSizeType)}
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {IMAGE_SIZE_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">提示词</div>
          <Textarea
            value={prompt}
            onChange={(event) => onPromptChange(event.target.value)}
            placeholder="请输入提示词"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-blue-200"
          />
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!prompt.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function GenerateCoverPanel({
  platform,
  count,
  description,
  isSubmitting,
  onPlatformChange,
  onCountChange,
  onDescriptionChange,
  onSubmit,
  onCancel,
}: {
  platform: CoverPlatformType;
  count: CoverCountType;
  description: string;
  isSubmitting?: boolean;
  onPlatformChange: (value: CoverPlatformType) => void;
  onCountChange: (value: CoverCountType) => void;
  onDescriptionChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-blue-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-cover-panel"
    >
      <div className="rounded-2xl border border-blue-100 bg-blue-50/70 px-4 py-3 text-blue-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <LayoutTemplate className="h-3.5 w-3.5" />
          <span>生成封面</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">
              投放平台
            </div>
            <Select
              value={platform}
              onValueChange={(value) =>
                onPlatformChange(value as CoverPlatformType)
              }
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {COVER_PLATFORM_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">
              生成数量
            </div>
            <Select
              value={count}
              onValueChange={(value) => onCountChange(value as CoverCountType)}
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {COVER_COUNT_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">封面描述</div>
          <Textarea
            value={description}
            onChange={(event) => onDescriptionChange(event.target.value)}
            placeholder="请输入封面描述"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-blue-200"
          />
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!description.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function GenerateStoryboardPanel({
  onSubmit,
  onCancel,
}: {
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-violet-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-storyboard-panel"
    >
      <div className="rounded-2xl border border-violet-100 bg-violet-50/70 px-4 py-3 text-violet-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Film className="h-3.5 w-3.5" />
          <span>生成分镜</span>
        </div>
      </div>

      <div className="mt-4 min-h-[112px]" />

      <div className="mt-4 flex items-center gap-3 border-t border-border/70 pt-4">
        <Button
          type="button"
          className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
          onClick={onSubmit}
        >
          一键生成
        </Button>
        <Button
          type="button"
          variant="secondary"
          className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
          onClick={onCancel}
        >
          取消
        </Button>
      </div>
    </div>
  );
}

function GenerateVideoAssetsPanel({
  model,
  version,
  ratio,
  duration,
  prompt,
  isSubmitting,
  onModelChange,
  onVersionChange,
  onRatioChange,
  onDurationChange,
  onPromptChange,
  onSubmit,
  onCancel,
}: {
  model: VideoAssetModelType;
  version: VideoAssetVersionType;
  ratio: VideoAssetRatioType;
  duration: VideoAssetDurationType;
  prompt: string;
  isSubmitting?: boolean;
  onModelChange: (value: VideoAssetModelType) => void;
  onVersionChange: (value: VideoAssetVersionType) => void;
  onRatioChange: (value: VideoAssetRatioType) => void;
  onDurationChange: (value: VideoAssetDurationType) => void;
  onPromptChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-blue-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-video-assets-panel"
    >
      <div className="rounded-2xl border border-blue-100 bg-blue-50/70 px-4 py-3 text-blue-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Clapperboard className="h-3.5 w-3.5" />
          <span>生成视频素材</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="grid grid-cols-3 gap-3">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">模型</div>
            <Select
              value={model}
              onValueChange={(value) =>
                onModelChange(value as VideoAssetModelType)
              }
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {VIDEO_ASSET_MODEL_OPTIONS.filter(
                  (option) => !option.disabled,
                ).map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">版本</div>
            <Select
              value={version}
              onValueChange={(value) =>
                onVersionChange(value as VideoAssetVersionType)
              }
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {VIDEO_ASSET_VERSION_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">比例</div>
            <Select
              value={ratio}
              onValueChange={(value) =>
                onRatioChange(value as VideoAssetRatioType)
              }
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {VIDEO_ASSET_RATIO_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="w-[88px] space-y-2">
          <div className="text-xs font-semibold text-foreground">时长 (秒)</div>
          <Select
            value={duration}
            onValueChange={(value) =>
              onDurationChange(value as VideoAssetDurationType)
            }
          >
            <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent side="bottom">
              {VIDEO_ASSET_DURATION_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">提示词</div>
          <div className="flex items-start gap-3">
            <div className="h-[84px] w-[60px] rounded-xl border border-slate-200 bg-slate-50 flex items-center justify-center text-slate-400">
              <Image className="h-5 w-5" />
            </div>
            <Textarea
              value={prompt}
              onChange={(event) => onPromptChange(event.target.value)}
              placeholder="请输入提示词"
              className="min-h-[84px] flex-1 resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-blue-200"
            />
          </div>
        </div>

        <div className="flex items-center gap-3">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!prompt.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function GenerateAIVideoPanel({
  scriptContent,
  isSubmitting,
  onScriptContentChange,
  onSubmit,
  onCancel,
}: {
  scriptContent: string;
  isSubmitting?: boolean;
  onScriptContentChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-blue-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-ai-video-panel"
    >
      <div className="rounded-2xl border border-blue-100 bg-blue-50/70 px-4 py-3 text-blue-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Video className="h-3.5 w-3.5" />
          <span>生成视频(非AI画面)</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">脚本内容</div>
          <Textarea
            value={scriptContent}
            onChange={(event) => onScriptContentChange(event.target.value)}
            placeholder="请输入脚本内容"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-blue-200"
          />
        </div>

        <div className="flex items-center gap-3 border-t border-border/70 pt-4">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!scriptContent.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function VoiceTonePickerDialog({
  open,
  activeTab,
  searchKeyword,
  selectedToneId,
  onOpenChange,
  onActiveTabChange,
  onSearchKeywordChange,
  onSelectTone,
}: {
  open: boolean;
  activeTab: VoiceoverToneTabType;
  searchKeyword: string;
  selectedToneId: VoiceoverToneId;
  onOpenChange: (open: boolean) => void;
  onActiveTabChange: (tab: VoiceoverToneTabType) => void;
  onSearchKeywordChange: (value: string) => void;
  onSelectTone: (toneId: VoiceoverToneId) => void;
}) {
  const normalizedKeyword = searchKeyword.trim().toLowerCase();
  const filteredTones = VOICEOVER_TONE_OPTIONS.filter((tone) => {
    if (!normalizedKeyword) {
      return true;
    }
    return tone.label.toLowerCase().includes(normalizedKeyword);
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="p-4 sm:max-w-[860px]">
        <div className="space-y-4" data-testid="workbench-voice-tone-dialog">
          <div className="flex items-center gap-2 rounded-xl bg-muted/40 px-3 py-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <input
              value={searchKeyword}
              onChange={(event) => onSearchKeywordChange(event.target.value)}
              placeholder="搜索音色"
              className="h-6 flex-1 border-0 bg-transparent text-sm outline-none"
            />
          </div>

          <div className="flex items-center gap-2">
            <button
              type="button"
              className={cn(
                "h-9 rounded-lg px-4 text-sm transition-colors",
                activeTab === "mine"
                  ? "bg-blue-50 text-blue-600"
                  : "bg-muted/50 text-muted-foreground hover:bg-muted",
              )}
              onClick={() => onActiveTabChange("mine")}
            >
              我的音色
            </button>
            <button
              type="button"
              className={cn(
                "h-9 rounded-lg px-4 text-sm transition-colors",
                activeTab === "library"
                  ? "bg-blue-50 text-blue-600"
                  : "bg-muted/50 text-muted-foreground hover:bg-muted",
              )}
              onClick={() => onActiveTabChange("library")}
            >
              素材库
            </button>
          </div>

          <div className="grid grid-cols-3 gap-3 pb-1">
            {filteredTones.map((tone) => {
              const selected = tone.id === selectedToneId;
              return (
                <button
                  key={tone.id}
                  type="button"
                  className={cn(
                    "h-20 rounded-xl border px-4 text-left transition-colors",
                    selected
                      ? "border-blue-400 bg-blue-50/60"
                      : "border-slate-200 bg-white hover:border-blue-200",
                  )}
                  onClick={() => {
                    onSelectTone(tone.id);
                    onOpenChange(false);
                  }}
                >
                  <div className="flex items-center gap-3">
                    <div className="h-12 w-12 rounded-full bg-emerald-200" />
                    <div className="min-w-0 flex-1">
                      <div className="truncate text-sm font-semibold text-foreground">
                        {tone.label}
                      </div>
                      <div className="mt-0.5 text-xs text-muted-foreground">
                        {tone.gender}
                      </div>
                    </div>
                    {selected ? (
                      <Check className="h-4 w-4 text-blue-500" />
                    ) : null}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function GenerateVoiceoverPanel({
  speed,
  toneId,
  prompt,
  isSubmitting,
  generatedAudioUrl,
  toneDialogOpen,
  toneDialogTab,
  toneDialogSearchKeyword,
  onSpeedChange,
  onPromptChange,
  onToneDialogOpenChange,
  onToneDialogTabChange,
  onToneDialogSearchKeywordChange,
  onToneSelect,
  onSubmit,
  onCancel,
}: {
  speed: VoiceoverSpeedType;
  toneId: VoiceoverToneId;
  prompt: string;
  isSubmitting?: boolean;
  generatedAudioUrl?: string;
  toneDialogOpen: boolean;
  toneDialogTab: VoiceoverToneTabType;
  toneDialogSearchKeyword: string;
  onSpeedChange: (value: VoiceoverSpeedType) => void;
  onPromptChange: (value: string) => void;
  onToneDialogOpenChange: (open: boolean) => void;
  onToneDialogTabChange: (tab: VoiceoverToneTabType) => void;
  onToneDialogSearchKeywordChange: (value: string) => void;
  onToneSelect: (toneId: VoiceoverToneId) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  const selectedToneLabel =
    VOICEOVER_TONE_OPTIONS.find((tone) => tone.id === toneId)?.label ??
    "请选择音色";

  return (
    <div
      className="col-span-2 rounded-2xl border border-pink-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-voiceover-panel"
    >
      <div className="rounded-2xl border border-pink-100 bg-pink-50/70 px-4 py-3 text-pink-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Mic className="h-3.5 w-3.5" />
          <span>生成配音</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">语速</div>
            <Select
              value={speed}
              onValueChange={(value) =>
                onSpeedChange(value as VoiceoverSpeedType)
              }
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {VOICEOVER_SPEED_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">
              选择音色
            </div>
            <Button
              type="button"
              variant="secondary"
              data-testid="workbench-voice-tone-trigger"
              className="h-9 w-[72px] justify-between rounded-xl bg-muted/60 px-3 text-muted-foreground hover:bg-muted"
              onClick={() => onToneDialogOpenChange(true)}
            >
              <span className="max-w-[34px] truncate">{selectedToneLabel}</span>
              <ChevronDown className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">提示词</div>
          <Textarea
            value={prompt}
            onChange={(event) => onPromptChange(event.target.value)}
            placeholder="请输入提示词"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-pink-200"
          />
        </div>

        <div className="flex items-center gap-3 border-t border-border/70 pt-4">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!prompt.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "生成中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>

        {generatedAudioUrl ? (
          <div className="rounded-xl border border-pink-100 bg-pink-50/40 px-3 py-2">
            <audio
              controls
              src={generatedAudioUrl}
              data-testid="workbench-voiceover-audio-preview"
              className="w-full"
            />
          </div>
        ) : null}
      </div>

      <VoiceTonePickerDialog
        open={toneDialogOpen}
        activeTab={toneDialogTab}
        searchKeyword={toneDialogSearchKeyword}
        selectedToneId={toneId}
        onOpenChange={onToneDialogOpenChange}
        onActiveTabChange={onToneDialogTabChange}
        onSearchKeywordChange={onToneDialogSearchKeywordChange}
        onSelectTone={onToneSelect}
      />
    </div>
  );
}

function GenerateBgmPanel({
  duration,
  prompt,
  isSubmitting,
  onDurationChange,
  onPromptChange,
  onSubmit,
  onCancel,
}: {
  duration: BgmDurationType;
  prompt: string;
  isSubmitting?: boolean;
  onDurationChange: (value: BgmDurationType) => void;
  onPromptChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-pink-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-bgm-panel"
    >
      <div className="rounded-2xl border border-pink-100 bg-pink-50/70 px-4 py-3 text-pink-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <Music4 className="h-3.5 w-3.5" />
          <span>生成BGM</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="w-[88px] space-y-2">
          <div className="text-xs font-semibold text-foreground">时长 (秒)</div>
          <Select
            value={duration}
            onValueChange={(value) =>
              onDurationChange(value as BgmDurationType)
            }
          >
            <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent side="bottom">
              {BGM_DURATION_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">提示词</div>
          <Textarea
            value={prompt}
            onChange={(event) => onPromptChange(event.target.value)}
            placeholder="请输入提示词"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-pink-200"
          />
        </div>

        <div className="flex items-center gap-3 border-t border-border/70 pt-4">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!prompt.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function GenerateSfxPanel({
  duration,
  prompt,
  isSubmitting,
  onDurationChange,
  onPromptChange,
  onSubmit,
  onCancel,
}: {
  duration: SfxDurationType;
  prompt: string;
  isSubmitting?: boolean;
  onDurationChange: (value: SfxDurationType) => void;
  onPromptChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-pink-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-sfx-panel"
    >
      <div className="rounded-2xl border border-pink-100 bg-pink-50/70 px-4 py-3 text-pink-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <AudioLines className="h-3.5 w-3.5" />
          <span>生成音效</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="w-[88px] space-y-2">
          <div className="text-xs font-semibold text-foreground">时长 (秒)</div>
          <Select
            value={duration}
            onValueChange={(value) =>
              onDurationChange(value as SfxDurationType)
            }
          >
            <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent side="bottom">
              {SFX_DURATION_OPTIONS.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div className="space-y-2">
          <div className="text-xs font-semibold text-foreground">提示词</div>
          <Textarea
            value={prompt}
            onChange={(event) => onPromptChange(event.target.value)}
            placeholder="请输入提示词"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-pink-200"
          />
        </div>

        <div className="flex items-center gap-3 border-t border-border/70 pt-4">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={!prompt.trim() || isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>
    </div>
  );
}

function PodcastVoicePickerDialog({
  open,
  speakerMode,
  searchKeyword,
  onOpenChange,
  onSpeakerModeChange,
  onSearchKeywordChange,
}: {
  open: boolean;
  speakerMode: PodcastSpeakerModeType;
  searchKeyword: string;
  onOpenChange: (open: boolean) => void;
  onSpeakerModeChange: (mode: PodcastSpeakerModeType) => void;
  onSearchKeywordChange: (value: string) => void;
}) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="p-4 sm:max-w-[860px]">
        <div className="space-y-4" data-testid="workbench-podcast-voice-dialog">
          <div className="flex items-center gap-2 rounded-xl bg-muted/40 px-3 py-2">
            <Search className="h-4 w-4 text-muted-foreground" />
            <input
              value={searchKeyword}
              onChange={(event) => onSearchKeywordChange(event.target.value)}
              placeholder="搜索音色"
              className="h-6 flex-1 border-0 bg-transparent text-sm outline-none"
            />
          </div>

          <div className="space-y-3">
            <div className="text-xl font-semibold text-foreground">
              选择模式
            </div>
            <div className="grid grid-cols-2 gap-4">
              <button
                type="button"
                className={cn(
                  "h-[80px] rounded-xl border px-4 text-base font-semibold transition-colors",
                  speakerMode === "dual"
                    ? "border-black text-foreground"
                    : "border-slate-200 text-muted-foreground",
                )}
                onClick={() => onSpeakerModeChange("dual")}
              >
                <span className="flex items-center justify-center gap-2">
                  <Users className="h-5 w-5" />
                  <span>双人</span>
                </span>
              </button>

              <button
                type="button"
                className={cn(
                  "h-[80px] rounded-xl border px-4 text-base font-semibold transition-colors",
                  speakerMode === "single"
                    ? "border-black text-foreground"
                    : "border-slate-200 text-muted-foreground",
                )}
                onClick={() => onSpeakerModeChange("single")}
              >
                <span className="flex items-center justify-center gap-2">
                  <User className="h-5 w-5" />
                  <span>单人</span>
                </span>
              </button>
            </div>
          </div>

          <div className="text-2xl font-semibold text-muted-foreground">
            {speakerMode === "dual" ? "选择 2 种音色" : "选择 1 种音色"}
          </div>

          <div className="min-h-[300px]" />
        </div>
      </DialogContent>
    </Dialog>
  );
}

function GeneratePodcastPanel({
  mode,
  prompt,
  isSubmitting,
  podcastVoiceDialogOpen,
  podcastSpeakerMode,
  podcastVoiceSearchKeyword,
  onModeChange,
  onPromptChange,
  onPodcastVoiceDialogOpenChange,
  onPodcastSpeakerModeChange,
  onPodcastVoiceSearchKeywordChange,
  onImportPrompt,
  onSubmit,
  onCancel,
}: {
  mode: PodcastModeType;
  prompt: string;
  isSubmitting?: boolean;
  podcastVoiceDialogOpen: boolean;
  podcastSpeakerMode: PodcastSpeakerModeType;
  podcastVoiceSearchKeyword: string;
  onModeChange: (value: PodcastModeType) => void;
  onPromptChange: (value: string) => void;
  onPodcastVoiceDialogOpenChange: (open: boolean) => void;
  onPodcastSpeakerModeChange: (mode: PodcastSpeakerModeType) => void;
  onPodcastVoiceSearchKeywordChange: (value: string) => void;
  onImportPrompt: () => void;
  onSubmit: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      className="col-span-2 rounded-2xl border border-pink-100 bg-white p-4 shadow-sm"
      data-testid="workbench-generate-podcast-panel"
    >
      <div className="rounded-2xl border border-pink-100 bg-pink-50/70 px-4 py-3 text-pink-500">
        <div className="flex items-center gap-2 text-xs font-semibold">
          <WandSparkles className="h-3.5 w-3.5" />
          <span>生成播客</span>
        </div>
      </div>

      <div className="mt-4 space-y-4">
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">
              播音音色
            </div>
            <Button
              type="button"
              variant="secondary"
              data-testid="workbench-podcast-voice-trigger"
              className="h-9 w-[72px] justify-between rounded-xl bg-muted/60 px-3 text-muted-foreground hover:bg-muted"
              onClick={() => onPodcastVoiceDialogOpenChange(true)}
            >
              <span className="max-w-[34px] truncate">选…</span>
              <ChevronDown className="h-3.5 w-3.5" />
            </Button>
          </div>

          <div className="space-y-2">
            <div className="text-xs font-semibold text-foreground">模式</div>
            <Select
              value={mode}
              onValueChange={(value) => onModeChange(value as PodcastModeType)}
            >
              <SelectTrigger className="h-9 rounded-xl bg-muted/60 border-0 text-muted-foreground shadow-none">
                <SelectValue />
              </SelectTrigger>
              <SelectContent side="bottom">
                {PODCAST_MODE_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <div className="text-xs font-semibold text-foreground">
              补充提示词
            </div>
            <button
              type="button"
              className="text-xs font-semibold text-blue-500 hover:text-blue-600"
              onClick={onImportPrompt}
            >
              一键导入
            </button>
          </div>
          <Textarea
            value={prompt}
            onChange={(event) => onPromptChange(event.target.value)}
            placeholder="你可以输入对播客主题的更多要求"
            className="min-h-[92px] resize-none rounded-2xl border-0 bg-slate-50 text-sm shadow-none focus-visible:ring-1 focus-visible:ring-pink-200"
          />
        </div>

        <div className="flex items-center gap-3 border-t border-border/70 pt-4">
          <Button
            type="button"
            className="h-10 flex-1 rounded-xl bg-slate-900 hover:bg-slate-800"
            disabled={isSubmitting}
            onClick={onSubmit}
          >
            {isSubmitting ? "提交中..." : "一键生成"}
          </Button>
          <Button
            type="button"
            variant="secondary"
            className="h-10 rounded-xl px-5 bg-slate-100 text-slate-600 hover:bg-slate-200"
            onClick={onCancel}
          >
            取消
          </Button>
        </div>
      </div>

      <PodcastVoicePickerDialog
        open={podcastVoiceDialogOpen}
        speakerMode={podcastSpeakerMode}
        searchKeyword={podcastVoiceSearchKeyword}
        onOpenChange={onPodcastVoiceDialogOpenChange}
        onSpeakerModeChange={onPodcastSpeakerModeChange}
        onSearchKeywordChange={onPodcastVoiceSearchKeywordChange}
      />
    </div>
  );
}

function GeneratedOutputsPanel({ items }: { items: GeneratedOutputItem[] }) {
  if (items.length === 0) {
    return (
      <div className="mt-auto min-h-[160px] rounded-2xl border border-dashed border-border/80 bg-muted/20 flex flex-col items-center justify-center gap-3 text-muted-foreground text-xs text-center px-4 py-6">
        <FileSearch className="h-6 w-6 opacity-50" />
        <p>生成的素材输出将保存在此处。</p>
      </div>
    );
  }

  return (
    <div className="mt-auto rounded-2xl border border-border/70 bg-muted/20 p-3">
      <div className="text-xs font-semibold text-foreground">生成输出</div>
      <div className="mt-2 max-h-[300px] space-y-2 overflow-y-auto pr-1">
        {items.map((item) => (
          <div
            key={item.id}
            data-testid="workbench-generated-output-item"
            className="rounded-xl border border-border/70 bg-background/95 p-2"
          >
            <div className="text-[12px] font-semibold text-foreground">
              {item.title}
            </div>
            <div className="mt-1 text-[11px] leading-relaxed text-muted-foreground">
              {item.detail}
            </div>
            {item.assetType === "image" && item.assetUrl ? (
              <img
                src={item.assetUrl}
                alt={item.title}
                className="mt-2 h-24 w-full rounded-md object-cover"
                loading="lazy"
                data-testid="workbench-generated-output-image"
              />
            ) : null}
            {item.assetType === "audio" && item.assetUrl ? (
              <audio
                controls
                src={item.assetUrl}
                className="mt-2 w-full"
                data-testid="workbench-generated-output-audio"
              />
            ) : null}
          </div>
        ))}
      </div>
    </div>
  );
}

function CapabilityPanel({
  onCollapse,
  projectId,
  onCreateContentFromPrompt,
  initialExpandedActionKey,
  onInitialExpandedActionConsumed,
}: {
  onCollapse: () => void;
  projectId?: string | null;
  onCreateContentFromPrompt?: (prompt: string) => Promise<void> | void;
  initialExpandedActionKey?: string | null;
  onInitialExpandedActionConsumed?: () => void;
}) {
  const [expandedActionKey, setExpandedActionKey] = useState<string | null>(
    null,
  );
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
  const [videoAssetModel, setVideoAssetModel] =
    useState<VideoAssetModelType>("keling");
  const [videoAssetVersion, setVideoAssetVersion] =
    useState<VideoAssetVersionType>("v2-1-master");
  const [videoAssetRatio, setVideoAssetRatio] =
    useState<VideoAssetRatioType>("16-9");
  const [videoAssetDuration, setVideoAssetDuration] =
    useState<VideoAssetDurationType>("5s");
  const [videoAssetPrompt, setVideoAssetPrompt] = useState("");
  const [aiVideoScriptContent, setAiVideoScriptContent] = useState("");
  const [voiceoverSpeed, setVoiceoverSpeed] =
    useState<VoiceoverSpeedType>("1.0x");
  const [voiceoverToneId, setVoiceoverToneId] =
    useState<VoiceoverToneId>("gaolengyujie");
  const [voiceoverPrompt, setVoiceoverPrompt] = useState("");
  const [voiceoverSubmitting, setVoiceoverSubmitting] = useState(false);
  const [voiceoverAudioUrl, setVoiceoverAudioUrl] = useState("");
  const [voiceToneDialogOpen, setVoiceToneDialogOpen] = useState(false);
  const [voiceToneDialogTab, setVoiceToneDialogTab] =
    useState<VoiceoverToneTabType>("library");
  const [voiceToneDialogSearchKeyword, setVoiceToneDialogSearchKeyword] =
    useState("");
  const [bgmDuration, setBgmDuration] = useState<BgmDurationType>("30s");
  const [bgmPrompt, setBgmPrompt] = useState("");
  const [bgmSubmitting, setBgmSubmitting] = useState(false);
  const [sfxDuration, setSfxDuration] = useState<SfxDurationType>("10s");
  const [sfxPrompt, setSfxPrompt] = useState("");
  const [sfxSubmitting, setSfxSubmitting] = useState(false);
  const [podcastMode, setPodcastMode] = useState<PodcastModeType>("deep");
  const [podcastPrompt, setPodcastPrompt] = useState("");
  const [podcastSubmitting, setPodcastSubmitting] = useState(false);
  const [podcastVoiceDialogOpen, setPodcastVoiceDialogOpen] = useState(false);
  const [podcastSpeakerMode, setPodcastSpeakerMode] =
    useState<PodcastSpeakerModeType>("dual");
  const [podcastVoiceSearchKeyword, setPodcastVoiceSearchKeyword] =
    useState("");
  const [videoProviders, setVideoProviders] = useState<VideoProviderOption[]>(
    [],
  );
  const [imageSubmitting, setImageSubmitting] = useState(false);
  const [coverSubmitting, setCoverSubmitting] = useState(false);
  const [videoAssetSubmitting, setVideoAssetSubmitting] = useState(false);
  const [aiVideoSubmitting, setAiVideoSubmitting] = useState(false);
  const [generatedOutputs, setGeneratedOutputs] = useState<
    GeneratedOutputItem[]
  >([]);
  const generatedOutputsRef = useRef<GeneratedOutputItem[]>([]);
  const { mediaDefaults } = useGlobalMediaGenerationDefaults();

  useEffect(() => {
    return () => {
      revokeObjectUrlIfNeeded(voiceoverAudioUrl);
    };
  }, [voiceoverAudioUrl]);

  useEffect(() => {
    generatedOutputsRef.current = generatedOutputs;
  }, [generatedOutputs]);

  useEffect(() => {
    if (!initialExpandedActionKey) {
      return;
    }
    setExpandedActionKey(initialExpandedActionKey);
    onInitialExpandedActionConsumed?.();
  }, [initialExpandedActionKey, onInitialExpandedActionConsumed]);

  useEffect(() => {
    return () => {
      for (const item of generatedOutputsRef.current) {
        revokeObjectUrlIfNeeded(item.assetUrl ?? "");
      }
    };
  }, []);

  useEffect(() => {
    let active = true;
    const loadVideoProviders = async () => {
      try {
        const allProviders = await apiKeyProviderApi.getProviders();
        if (!active) {
          return;
        }

        const availableProviders = allProviders
          .filter(
            (provider) =>
              provider.enabled &&
              provider.api_key_count > 0 &&
              isVideoProvider(provider.id),
          )
          .map((provider) => ({
            id: provider.id,
            customModels: provider.custom_models ?? [],
          }));
        setVideoProviders(availableProviders);
      } catch (error) {
        console.error("[WorkbenchRightRail] 加载视频 Provider 失败:", error);
        if (active) {
          setVideoProviders([]);
        }
      }
    };

    void loadVideoProviders();
    return () => {
      active = false;
    };
  }, []);

  const handleSubmitPrompt = async (prompt: string): Promise<boolean> => {
    const normalizedPrompt = prompt.trim();
    if (!normalizedPrompt) {
      return false;
    }

    if (!onCreateContentFromPrompt) {
      toast.error("当前工作区暂不支持该操作");
      return false;
    }

    try {
      await onCreateContentFromPrompt(normalizedPrompt);
      return true;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`提交失败：${message}`);
      return false;
    }
  };

  const appendGeneratedOutput = (item: GeneratedOutputItem) => {
    setGeneratedOutputs((previous) => [item, ...previous].slice(0, 20));
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

  const loadCurrentProject = async () => {
    if (!projectId) {
      return null;
    }

    return invoke<Project | null>("workspace_get", {
      id: projectId,
    });
  };

  const resolveImageProviderAndModel = async (modelType: ImageModelType) => {
    const [allProviders, project] = await Promise.all([
      apiKeyProviderApi.getProviders(),
      loadCurrentProject(),
    ]);
    const imageProviders: ImageProviderOption[] = allProviders
      .filter(
        (provider) =>
          provider.enabled &&
          provider.api_key_count > 0 &&
          isImageProvider(provider.id, provider.type),
      )
      .map((provider) => ({
        id: provider.id,
        type: provider.type,
        apiHost: provider.api_host,
        customModels: provider.custom_models ?? [],
      }));

    const imagePreference = resolveMediaGenerationPreference(
      project?.settings?.imageGeneration,
      mediaDefaults.image,
    );
    const preferredProviderId =
      imagePreference.preferredProviderId?.trim() || "";
    const preferredModelId = imagePreference.preferredModelId?.trim() || "";
    const allowFallback = imagePreference.allowFallback;
    const preferenceSourceLabel =
      imagePreference.source === "project" ? "项目" : "全局默认";

    if (preferredProviderId) {
      const preferredProvider = findImageProviderById(
        imageProviders,
        preferredProviderId,
      );

      if (!preferredProvider) {
        if (!allowFallback) {
          throw new Error(
            `${preferenceSourceLabel}已指定图片服务 ${preferredProviderId}，但当前不可用，请前往设置调整`,
          );
        }
      } else {
        const preferredProviderModels = getImageModelIdsForProvider(
          preferredProvider.id,
          preferredProvider.type,
          preferredProvider.customModels,
        );
        const preferredModel =
          preferredModelId ||
          pickImageModelBySelection(preferredProviderModels, modelType);
        const preferredApiKey = await apiKeyProviderApi.getNextApiKey(
          preferredProvider.id,
        );

        if (preferredApiKey) {
          return {
            provider: preferredProvider,
            model: preferredModel,
            apiKey: preferredApiKey,
          };
        }

        if (!allowFallback) {
          throw new Error(
            `${preferenceSourceLabel}已指定图片服务 ${preferredProviderId}，但当前没有可用 API Key，请前往设置或凭证池调整`,
          );
        }
      }
    }

    const provider = findImageProviderForSelection(imageProviders, modelType);
    if (!provider) {
      throw new Error("未找到可用图片服务，请先在设置中配置 Provider");
    }

    const model = pickImageModelBySelection(
      getImageModelIdsForProvider(
        provider.id,
        provider.type,
        provider.customModels,
      ),
      modelType,
    );
    if (!model) {
      throw new Error("未能解析图片模型，请检查 Provider 模型配置");
    }

    const apiKey = await apiKeyProviderApi.getNextApiKey(provider.id);
    if (!apiKey) {
      throw new Error("该图片服务没有可用 API Key，请先在凭证池中添加");
    }

    return { provider, model, apiKey };
  };

  const requestImageGeneration = async ({
    provider,
    apiKey,
    model,
    prompt,
    size,
    count,
  }: {
    provider: ImageProviderOption;
    apiKey: string;
    model: string;
    prompt: string;
    size: string;
    count: number;
  }): Promise<string[]> => {
    const endpoint = buildProviderEndpoint(
      provider.apiHost,
      "/v1/images/generations",
    );
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${apiKey}`,
      },
      body: JSON.stringify({
        model,
        prompt,
        n: Math.max(1, Math.min(count, 4)),
        size,
      }),
    });
    const rawText = await response.text();
    const payload = tryParseJsonText(rawText);

    if (!response.ok) {
      const messageFromPayload = (
        payload?.error as { message?: string } | undefined
      )?.message;
      const message =
        typeof messageFromPayload === "string" &&
        messageFromPayload.trim().length > 0
          ? messageFromPayload
          : rawText.slice(0, 200);
      throw new Error(message || `请求失败: ${response.status}`);
    }

    const imageUrls = extractImageUrlsFromResponse(payload);
    if (imageUrls.length === 0) {
      throw new Error("图片服务返回成功但没有可用图片");
    }
    return imageUrls;
  };

  const requestAudioGeneration = async ({
    provider,
    apiKey,
    model,
    prompt,
    durationSeconds,
  }: {
    provider: TtsProviderOption;
    apiKey: string;
    model: string;
    prompt: string;
    durationSeconds: number;
  }): Promise<string> => {
    const attempts: Array<{
      endpoint: string;
      body: Record<string, unknown>;
    }> = [
      {
        endpoint: buildProviderEndpoint(
          provider.apiHost,
          "/v1/audio/generations",
        ),
        body: {
          model,
          prompt,
          duration: durationSeconds,
          format: "mp3",
        },
      },
      {
        endpoint: buildProviderEndpoint(provider.apiHost, "/v1/audio/speech"),
        body: {
          model,
          voice: "alloy",
          input: prompt,
          speed: 1,
        },
      },
    ];

    let lastError = "";

    for (const attempt of attempts) {
      try {
        const response = await fetch(attempt.endpoint, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${apiKey}`,
          },
          body: JSON.stringify(attempt.body),
        });

        if (!response.ok) {
          const rawText = await response.text();
          const payload = tryParseJsonText(rawText);
          const messageFromPayload = (
            payload?.error as { message?: string } | undefined
          )?.message;
          lastError =
            typeof messageFromPayload === "string" &&
            messageFromPayload.trim().length > 0
              ? messageFromPayload
              : rawText.slice(0, 200);
          continue;
        }

        const contentType =
          response.headers?.get?.("content-type")?.toLowerCase() ?? "";
        if (contentType.includes("application/json")) {
          const rawText = await response.text();
          const payload = tryParseJsonText(rawText);
          const audioUrl = extractAudioUrlFromResponse(payload);
          if (audioUrl) {
            return audioUrl;
          }
          lastError = "音频服务返回成功但没有可用音频地址";
          continue;
        }

        const audioBlob = await response.blob();
        if (audioBlob.size <= 0) {
          lastError = "音频服务返回成功但音频为空";
          continue;
        }

        if (
          typeof URL !== "undefined" &&
          typeof URL.createObjectURL === "function"
        ) {
          return URL.createObjectURL(audioBlob);
        }

        const dataUrl = await convertBlobToDataUrl(audioBlob);
        if (dataUrl) {
          return dataUrl;
        }
        lastError = "当前环境不支持音频预览";
      } catch (error) {
        lastError = error instanceof Error ? error.message : String(error);
      }
    }

    throw new Error(lastError || "音频生成失败");
  };

  const handleSubmitImageTask = async () => {
    const normalizedPrompt = imagePrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    setImageSubmitting(true);
    try {
      const { provider, model, apiKey } =
        await resolveImageProviderAndModel(imageModel);
      const size = mapImageSizeTypeToResolution(imageSize);
      const images = await requestImageGeneration({
        provider,
        apiKey,
        model,
        prompt: normalizedPrompt,
        size,
        count: 1,
      });
      appendGeneratedOutput({
        id: `image-${Date.now()}`,
        title: "图片生成成功",
        detail: `${provider.id} · ${model} · ${size}`,
        assetType: "image",
        assetUrl: images[0],
      });
      toast.success(`图片已生成（${provider.id} · ${model}）`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`图片生成失败：${message}`);
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
      const { provider, model, apiKey } =
        await resolveImageProviderAndModel(imageModel);
      const size = mapCoverPlatformToResolution(coverPlatform);
      const imageCount = Number.parseInt(coverCount, 10);
      const coverPrompt = `请生成${getOptionLabel(
        COVER_PLATFORM_OPTIONS,
        coverPlatform,
      )}平台封面图。要求：${normalizedPrompt}`;
      const images = await requestImageGeneration({
        provider,
        apiKey,
        model,
        prompt: coverPrompt,
        size,
        count: Number.isFinite(imageCount) ? imageCount : 1,
      });
      appendGeneratedOutput({
        id: `cover-${Date.now()}`,
        title: "封面生成成功",
        detail: `${provider.id} · ${model} · ${size} · ${images.length} 张`,
        assetType: "image",
        assetUrl: images[0],
      });
      toast.success(
        `封面已生成 ${images.length} 张（${provider.id} · ${model}）`,
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`封面生成失败：${message}`);
    } finally {
      setCoverSubmitting(false);
    }
  };

  const resolveTtsProviderAndModel = async () => {
    const [allProviders, project] = await Promise.all([
      apiKeyProviderApi.getProviders(),
      loadCurrentProject(),
    ]);
    const ttsProviders: TtsProviderOption[] = allProviders
      .filter(
        (provider) =>
          provider.enabled &&
          provider.api_key_count > 0 &&
          isTtsProvider(provider.id, provider.type),
      )
      .map((provider) => ({
        id: provider.id,
        type: provider.type,
        apiHost: provider.api_host,
        customModels: provider.custom_models ?? [],
      }));

    const voicePreference = resolveMediaGenerationPreference(
      project?.settings?.voiceGeneration,
      mediaDefaults.voice,
    );
    const preferredProviderId =
      voicePreference.preferredProviderId?.trim() || "";
    const preferredModelId = voicePreference.preferredModelId?.trim() || "";
    const allowFallback = voicePreference.allowFallback;
    const preferenceSourceLabel =
      voicePreference.source === "project" ? "项目" : "全局默认";

    if (preferredProviderId) {
      const preferredProvider = findMediaProviderById(
        ttsProviders,
        preferredProviderId,
      );

      if (!preferredProvider) {
        if (!allowFallback) {
          throw new Error(
            `${preferenceSourceLabel}已指定语音服务 ${preferredProviderId}，但当前不可用，请前往设置调整`,
          );
        }
      } else {
        const preferredModel =
          preferredModelId ||
          pickTtsModel(getTtsModelsForProvider(preferredProvider.customModels));
        const preferredApiKey = await apiKeyProviderApi.getNextApiKey(
          preferredProvider.id,
        );

        if (preferredApiKey) {
          return {
            provider: preferredProvider,
            model: preferredModel,
            apiKey: preferredApiKey,
          };
        }

        if (!allowFallback) {
          throw new Error(
            `${preferenceSourceLabel}已指定语音服务 ${preferredProviderId}，但当前没有可用 API Key，请前往设置或凭证池调整`,
          );
        }
      }
    }

    const provider = findTtsProviderForSelection(ttsProviders);
    if (!provider) {
      throw new Error("未找到可用配音服务，请先在设置中配置 Provider");
    }

    const model = pickTtsModel(getTtsModelsForProvider(provider.customModels));
    const apiKey = await apiKeyProviderApi.getNextApiKey(provider.id);
    if (!apiKey) {
      throw new Error("该配音服务没有可用 API Key，请先在凭证池中添加");
    }

    return { provider, model, apiKey };
  };

  const resolveVideoProviderAndModel = async () => {
    const project = await loadCurrentProject();
    const videoPreference = resolveMediaGenerationPreference(
      project?.settings?.videoGeneration,
      mediaDefaults.video,
    );
    const preferredProviderId =
      videoPreference.preferredProviderId?.trim() || "";
    const preferredModelId = videoPreference.preferredModelId?.trim() || "";
    const allowFallback = videoPreference.allowFallback;
    const preferenceSourceLabel =
      videoPreference.source === "project" ? "项目" : "全局默认";

    let provider = preferredProviderId
      ? findMediaProviderById(videoProviders, preferredProviderId)
      : null;

    if (!provider && preferredProviderId && !allowFallback) {
      throw new Error(
        `${preferenceSourceLabel}已指定视频服务 ${preferredProviderId}，但当前不可用，请前往设置调整`,
      );
    }

    if (!provider) {
      provider = findVideoProviderForSelection(videoProviders, videoAssetModel);
    }

    if (!provider) {
      throw new Error("未找到可用视频服务，请先在设置中配置 Provider");
    }

    const providerModels = getVideoModelsForProvider(
      provider.id,
      provider.customModels,
    );
    if (providerModels.length === 0) {
      if (
        preferredProviderId &&
        provider.id === preferredProviderId &&
        !allowFallback
      ) {
        throw new Error(
          preferenceSourceLabel +
            "指定的视频服务没有可用模型，请前往设置或凭证池调整",
        );
      }
      throw new Error("当前视频服务没有可用模型，请先补充模型配置");
    }

    const model =
      preferredProviderId &&
      provider.id === preferredProviderId &&
      preferredModelId
        ? preferredModelId
        : pickVideoModelByVersion(providerModels, videoAssetVersion);
    if (!model) {
      throw new Error("未能解析所选模型，请检查 Provider 模型配置");
    }

    return { provider, model };
  };

  const handleSubmitVoiceoverTask = async () => {
    const normalizedPrompt = voiceoverPrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    setVoiceoverSubmitting(true);
    try {
      const { provider, model, apiKey } = await resolveTtsProviderAndModel();
      const endpoint = buildProviderEndpoint(
        provider.apiHost,
        "/v1/audio/speech",
      );
      const voice = mapToneToTtsVoice(voiceoverToneId);
      const response = await fetch(endpoint, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${apiKey}`,
        },
        body: JSON.stringify({
          model,
          voice,
          input: normalizedPrompt,
          speed: parseVoiceSpeed(voiceoverSpeed),
        }),
      });

      if (!response.ok) {
        const rawText = await response.text();
        const payload = tryParseJsonText(rawText);
        const messageFromPayload = (
          payload?.error as { message?: string } | undefined
        )?.message;
        const message =
          typeof messageFromPayload === "string" &&
          messageFromPayload.trim().length > 0
            ? messageFromPayload
            : rawText.slice(0, 200);
        throw new Error(message || `请求失败: ${response.status}`);
      }

      const audioBlob = await response.blob();
      if (audioBlob.size <= 0) {
        throw new Error("配音服务返回成功但音频为空");
      }

      revokeObjectUrlIfNeeded(voiceoverAudioUrl);
      if (
        typeof URL !== "undefined" &&
        typeof URL.createObjectURL === "function"
      ) {
        const audioUrl = URL.createObjectURL(audioBlob);
        setVoiceoverAudioUrl(audioUrl);
      } else {
        setVoiceoverAudioUrl("");
      }

      appendGeneratedOutput({
        id: `voiceover-${Date.now()}`,
        title: "配音生成成功",
        detail: `${provider.id} · ${model} · 语速 ${voiceoverSpeed}`,
      });
      toast.success(`配音已生成（${provider.id} · ${model}）`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`配音生成失败：${message}`);
    } finally {
      setVoiceoverSubmitting(false);
    }
  };

  const buildBgmPrompt = () => {
    const durationLabel = getOptionLabel(BGM_DURATION_OPTIONS, bgmDuration);
    return `请生成纯背景音乐（不要人声）：
- 时长：${durationLabel}
- 提示词：${bgmPrompt.trim()}
请输出可直接用于视频配乐的音频。`;
  };

  const buildSfxPrompt = () => {
    const durationLabel = getOptionLabel(SFX_DURATION_OPTIONS, sfxDuration);
    return `请生成可直接使用的短音效（无需人声旁白）：
- 时长：${durationLabel}
- 提示词：${sfxPrompt.trim()}
请输出单段音效音频。`;
  };

  const buildPodcastPrompt = () => {
    const modeLabel = getOptionLabel(PODCAST_MODE_OPTIONS, podcastMode);
    const speakerModeLabel = podcastSpeakerMode === "dual" ? "双人" : "单人";
    return `请根据以下参数生成播客脚本：
- 模式：${modeLabel}
- 播音模式：${speakerModeLabel}
- 补充提示词：${podcastPrompt.trim()}

请输出分章节播客大纲与可直接录制的主持人台词。`;
  };

  const handleSubmitBgmTask = async () => {
    const normalizedPrompt = bgmPrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    setBgmSubmitting(true);
    try {
      const { provider, model, apiKey } = await resolveTtsProviderAndModel();
      const audioUrl = await requestAudioGeneration({
        provider,
        apiKey,
        model,
        prompt: buildBgmPrompt(),
        durationSeconds: parseSimpleDuration(bgmDuration, 30),
      });
      appendGeneratedOutput({
        id: `bgm-${Date.now()}`,
        title: "BGM 生成成功",
        detail: `${provider.id} · ${model} · ${getOptionLabel(BGM_DURATION_OPTIONS, bgmDuration)}`,
        assetType: "audio",
        assetUrl: audioUrl,
      });
      toast.success(`BGM 已生成（${provider.id} · ${model}）`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`BGM 生成失败：${message}`);
    } finally {
      setBgmSubmitting(false);
    }
  };

  const handleSubmitSfxTask = async () => {
    const normalizedPrompt = sfxPrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    setSfxSubmitting(true);
    try {
      const { provider, model, apiKey } = await resolveTtsProviderAndModel();
      const audioUrl = await requestAudioGeneration({
        provider,
        apiKey,
        model,
        prompt: buildSfxPrompt(),
        durationSeconds: parseSimpleDuration(sfxDuration, 10),
      });
      appendGeneratedOutput({
        id: `sfx-${Date.now()}`,
        title: "音效生成成功",
        detail: `${provider.id} · ${model} · ${getOptionLabel(SFX_DURATION_OPTIONS, sfxDuration)}`,
        assetType: "audio",
        assetUrl: audioUrl,
      });
      toast.success(`音效已生成（${provider.id} · ${model}）`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`音效生成失败：${message}`);
    } finally {
      setSfxSubmitting(false);
    }
  };

  const handleSubmitPodcastTask = async () => {
    setPodcastSubmitting(true);
    try {
      const submitted = await handleSubmitPrompt(buildPodcastPrompt());
      if (!submitted) {
        return;
      }
      appendGeneratedOutput({
        id: `podcast-${Date.now()}`,
        title: "播客脚本任务已提交",
        detail: `${getOptionLabel(PODCAST_MODE_OPTIONS, podcastMode)} · ${
          podcastSpeakerMode === "dual" ? "双人播音" : "单人播音"
        }`,
      });
    } finally {
      setPodcastSubmitting(false);
    }
  };

  const handleSubmitVideoAssetsTask = async () => {
    const normalizedPrompt = videoAssetPrompt.trim();
    if (!normalizedPrompt) {
      return;
    }

    if (!projectId) {
      toast.error("请先选择项目后再生成视频素材");
      return;
    }

    setVideoAssetSubmitting(true);
    try {
      const { provider, model: selectedModel } =
        await resolveVideoProviderAndModel();

      const createdTask = await videoGenerationApi.createTask({
        projectId,
        providerId: provider.id,
        model: selectedModel,
        prompt: normalizedPrompt,
        aspectRatio: getOptionLabel(VIDEO_ASSET_RATIO_OPTIONS, videoAssetRatio),
        resolution: "720p",
        duration: parseVideoDuration(videoAssetDuration),
      });
      appendGeneratedOutput({
        id: `video-assets-${createdTask.id}`,
        title: "视频素材任务已提交",
        detail: `${provider.id} · ${selectedModel} · 任务 ${createdTask.id}`,
      });
      toast.success(`视频素材任务已提交（${provider.id} · ${selectedModel}）`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`视频素材任务提交失败：${message}`);
    } finally {
      setVideoAssetSubmitting(false);
    }
  };

  const handleSubmitAIVideoTask = async () => {
    const normalizedScript = aiVideoScriptContent.trim();
    if (!normalizedScript) {
      return;
    }

    if (!projectId) {
      toast.error("请先选择项目后再生成视频");
      return;
    }

    setAiVideoSubmitting(true);
    try {
      const { provider, model: selectedModel } =
        await resolveVideoProviderAndModel();

      const createdTask = await videoGenerationApi.createTask({
        projectId,
        providerId: provider.id,
        model: selectedModel,
        prompt: normalizedScript,
        aspectRatio: getOptionLabel(VIDEO_ASSET_RATIO_OPTIONS, videoAssetRatio),
        resolution: "720p",
        duration: parseVideoDuration(videoAssetDuration),
      });
      appendGeneratedOutput({
        id: `video-script-${createdTask.id}`,
        title: "非AI画面视频任务已提交",
        detail: `${provider.id} · ${selectedModel} · 任务 ${createdTask.id}`,
      });
      toast.success(
        `非 AI 画面视频任务已提交（${provider.id} · ${selectedModel}）`,
      );
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      toast.error(`视频任务提交失败：${message}`);
    } finally {
      setAiVideoSubmitting(false);
    }
  };

  return (
    <aside
      className="w-[320px] min-w-[320px] border-l bg-background/95 flex flex-col"
      data-testid="workbench-right-rail-expanded"
    >
      <div className="flex items-center justify-end px-3 py-2 border-b bg-background/96">
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="h-8 w-8 rounded-md"
                onClick={onCollapse}
                title="折叠能力面板"
              >
                <PanelRightClose size={16} />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="left">
              <p>折叠能力面板</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      </div>

      <div className="flex-1 min-h-0 overflow-y-auto px-3 py-3 flex flex-col gap-4">
        {CAPABILITY_SECTIONS.map((section) => {
          const expandedActionInSection = section.items.find(
            (item) => item.key === expandedActionKey,
          )?.key;

          return (
            <section key={section.key} className="space-y-2">
              <div
                className={cn(
                  "text-xs font-semibold flex items-center gap-2",
                  SECTION_TONE_CLASS[section.tone],
                )}
              >
                <Sparkles className="h-3.5 w-3.5" />
                <span>{section.title}</span>
              </div>

              <div className="grid grid-cols-2 gap-2">
                {section.items.map((item) => {
                  if (item.key === expandedActionInSection) {
                    if (item.key === "search-material") {
                      return (
                        <SearchMaterialPanel
                          key={`panel-${item.key}`}
                          resourceType={searchResourceType}
                          searchQuery={searchQuery}
                          isSubmitting={searchMaterialSubmitting}
                          resultSummary={searchMaterialResultSummary}
                          onResourceTypeChange={setSearchResourceType}
                          onSearchQueryChange={setSearchQuery}
                          onSubmit={() => {
                            void handleSubmitSearchMaterial();
                          }}
                          onCancel={() => {
                            setSearchMaterialResultSummary("");
                            setExpandedActionKey(null);
                          }}
                        />
                      );
                    }

                    if (item.key === "generate-title") {
                      return (
                        <GenerateTitlePanel
                          key={`panel-${item.key}`}
                          requirement={titleRequirement}
                          onRequirementChange={setTitleRequirement}
                          onSubmit={() => {
                            void handleSubmitPrompt(`请根据以下要求生成 10 个标题候选，标题需风格多样且避免重复：
${titleRequirement.trim()}`);
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-image") {
                      return (
                        <GenerateImagePanel
                          key={`panel-${item.key}`}
                          model={imageModel}
                          size={imageSize}
                          prompt={imagePrompt}
                          isSubmitting={imageSubmitting}
                          onModelChange={setImageModel}
                          onSizeChange={setImageSize}
                          onPromptChange={setImagePrompt}
                          onSubmit={() => {
                            void handleSubmitImageTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-cover") {
                      return (
                        <GenerateCoverPanel
                          key={`panel-${item.key}`}
                          platform={coverPlatform}
                          count={coverCount}
                          description={coverDescription}
                          isSubmitting={coverSubmitting}
                          onPlatformChange={setCoverPlatform}
                          onCountChange={setCoverCount}
                          onDescriptionChange={setCoverDescription}
                          onSubmit={() => {
                            void handleSubmitCoverTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-storyboard") {
                      return (
                        <GenerateStoryboardPanel
                          key={`panel-${item.key}`}
                          onSubmit={() => {
                            void handleSubmitPrompt(
                              "请根据当前项目主题生成完整分镜脚本，包含镜头序号、画面内容、台词/旁白、时长与运镜建议。",
                            );
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-video-assets") {
                      return (
                        <GenerateVideoAssetsPanel
                          key={`panel-${item.key}`}
                          model={videoAssetModel}
                          version={videoAssetVersion}
                          ratio={videoAssetRatio}
                          duration={videoAssetDuration}
                          prompt={videoAssetPrompt}
                          isSubmitting={videoAssetSubmitting}
                          onModelChange={setVideoAssetModel}
                          onVersionChange={setVideoAssetVersion}
                          onRatioChange={setVideoAssetRatio}
                          onDurationChange={setVideoAssetDuration}
                          onPromptChange={setVideoAssetPrompt}
                          onSubmit={() => {
                            void handleSubmitVideoAssetsTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-ai-video") {
                      return (
                        <GenerateAIVideoPanel
                          key={`panel-${item.key}`}
                          scriptContent={aiVideoScriptContent}
                          isSubmitting={aiVideoSubmitting}
                          onScriptContentChange={setAiVideoScriptContent}
                          onSubmit={() => {
                            void handleSubmitAIVideoTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-voiceover") {
                      return (
                        <GenerateVoiceoverPanel
                          key={`panel-${item.key}`}
                          speed={voiceoverSpeed}
                          toneId={voiceoverToneId}
                          prompt={voiceoverPrompt}
                          isSubmitting={voiceoverSubmitting}
                          generatedAudioUrl={voiceoverAudioUrl}
                          toneDialogOpen={voiceToneDialogOpen}
                          toneDialogTab={voiceToneDialogTab}
                          toneDialogSearchKeyword={voiceToneDialogSearchKeyword}
                          onSpeedChange={setVoiceoverSpeed}
                          onPromptChange={setVoiceoverPrompt}
                          onToneDialogOpenChange={setVoiceToneDialogOpen}
                          onToneDialogTabChange={setVoiceToneDialogTab}
                          onToneDialogSearchKeywordChange={
                            setVoiceToneDialogSearchKeyword
                          }
                          onToneSelect={setVoiceoverToneId}
                          onSubmit={() => {
                            void handleSubmitVoiceoverTask();
                          }}
                          onCancel={() => {
                            setVoiceToneDialogOpen(false);
                            setExpandedActionKey(null);
                          }}
                        />
                      );
                    }

                    if (item.key === "generate-bgm") {
                      return (
                        <GenerateBgmPanel
                          key={`panel-${item.key}`}
                          duration={bgmDuration}
                          prompt={bgmPrompt}
                          isSubmitting={bgmSubmitting}
                          onDurationChange={setBgmDuration}
                          onPromptChange={setBgmPrompt}
                          onSubmit={() => {
                            void handleSubmitBgmTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-sfx") {
                      return (
                        <GenerateSfxPanel
                          key={`panel-${item.key}`}
                          duration={sfxDuration}
                          prompt={sfxPrompt}
                          isSubmitting={sfxSubmitting}
                          onDurationChange={setSfxDuration}
                          onPromptChange={setSfxPrompt}
                          onSubmit={() => {
                            void handleSubmitSfxTask();
                          }}
                          onCancel={() => setExpandedActionKey(null)}
                        />
                      );
                    }

                    if (item.key === "generate-podcast") {
                      return (
                        <GeneratePodcastPanel
                          key={`panel-${item.key}`}
                          mode={podcastMode}
                          prompt={podcastPrompt}
                          isSubmitting={podcastSubmitting}
                          podcastVoiceDialogOpen={podcastVoiceDialogOpen}
                          podcastSpeakerMode={podcastSpeakerMode}
                          podcastVoiceSearchKeyword={podcastVoiceSearchKeyword}
                          onModeChange={setPodcastMode}
                          onPromptChange={setPodcastPrompt}
                          onPodcastVoiceDialogOpenChange={
                            setPodcastVoiceDialogOpen
                          }
                          onPodcastSpeakerModeChange={setPodcastSpeakerMode}
                          onPodcastVoiceSearchKeywordChange={
                            setPodcastVoiceSearchKeyword
                          }
                          onImportPrompt={() =>
                            setPodcastPrompt(PODCAST_QUICK_IMPORT_PROMPT)
                          }
                          onSubmit={() => {
                            void handleSubmitPodcastTask();
                          }}
                          onCancel={() => {
                            setPodcastVoiceDialogOpen(false);
                            setExpandedActionKey(null);
                          }}
                        />
                      );
                    }
                  }

                  const Icon = item.icon;
                  return (
                    <button
                      key={item.key}
                      type="button"
                      data-testid={`workbench-right-rail-action-${item.key}`}
                      className={cn(
                        "h-[58px] rounded-xl border px-3 py-2 text-left transition-colors",
                        CARD_TONE_CLASS[item.tone],
                      )}
                      onClick={() => {
                        if (
                          item.key === "search-material" ||
                          item.key === "generate-title" ||
                          item.key === "generate-image" ||
                          item.key === "generate-cover" ||
                          item.key === "generate-storyboard" ||
                          item.key === "generate-video-assets" ||
                          item.key === "generate-ai-video" ||
                          item.key === "generate-voiceover" ||
                          item.key === "generate-bgm" ||
                          item.key === "generate-sfx" ||
                          item.key === "generate-podcast"
                        ) {
                          setVoiceToneDialogOpen(false);
                          setPodcastVoiceDialogOpen(false);
                          setExpandedActionKey((previous) =>
                            previous === item.key ? null : item.key,
                          );
                        }
                      }}
                    >
                      <div className="flex items-center gap-2 text-xs font-medium leading-none">
                        <Icon className="h-3.5 w-3.5" />
                        <span className="leading-tight">{item.label}</span>
                      </div>
                    </button>
                  );
                })}
              </div>
            </section>
          );
        })}

        <GeneratedOutputsPanel items={generatedOutputs} />
      </div>
    </aside>
  );
}

const COLLAPSED_ACTION_TONE_CLASS: Record<CapabilityItem["tone"], string> = {
  violet:
    "border-violet-100 text-violet-500 hover:border-violet-200 hover:bg-violet-50/80",
  blue: "border-blue-100 text-blue-500 hover:border-blue-200 hover:bg-blue-50/80",
  pink: "border-pink-100 text-pink-500 hover:border-pink-200 hover:bg-pink-50/80",
};

function CollapsedRail({
  onExpand,
  onExpandToAction,
}: {
  onExpand: () => void;
  onExpandToAction: (actionKey: string) => void;
}) {
  return (
    <aside
      className="relative z-20 w-14 min-w-14 overflow-hidden border-l bg-background/95 flex flex-col items-center py-3 gap-2"
      data-testid="workbench-right-rail-collapsed"
    >
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-8 w-8 rounded-md"
              data-testid="workbench-right-rail-collapsed-expand"
              onClick={onExpand}
              title="展开能力面板"
            >
              <PanelRightOpen size={16} />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="left">
            <p>展开能力面板</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>

      <div className="flex-1 min-h-0 overflow-y-auto px-2 py-1 flex flex-col gap-2">
        {CAPABILITY_SECTIONS.flatMap((section) => section.items).map((item) => {
          const Icon = item.icon;
          return (
            <TooltipProvider key={item.key}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    type="button"
                    data-testid={`workbench-right-rail-collapsed-action-${item.key}`}
                    className={cn(
                      "h-8 w-8 rounded-lg border bg-white/90 transition-colors flex items-center justify-center",
                      COLLAPSED_ACTION_TONE_CLASS[item.tone],
                    )}
                    title={item.label}
                    onClick={() => onExpandToAction(item.key)}
                  >
                    <Icon className="h-3.5 w-3.5" />
                  </button>
                </TooltipTrigger>
                <TooltipContent side="left">
                  <p>{item.label}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          );
        })}
      </div>
    </aside>
  );
}

function NonCreateRail({
  onBackToCreateView,
}: Pick<WorkbenchRightRailProps, "onBackToCreateView">) {
  return (
    <aside className="w-14 min-w-14 border-l bg-background/95 flex flex-col items-center py-3 gap-2">
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-9 w-9"
              onClick={onBackToCreateView}
              title="返回创作视图"
            >
              <Bot className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="left">
            <p>返回创作视图</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </aside>
  );
}

export function WorkbenchRightRail({
  shouldRender,
  isCreateWorkspaceView,
  projectId,
  onBackToCreateView,
  onCreateContentFromPrompt,
}: WorkbenchRightRailProps) {
  const [collapsed, setCollapsed] = useState(false);
  const [pendingExpandedActionKey, setPendingExpandedActionKey] = useState<
    string | null
  >(null);
  const contentReviewRailState = useWorkbenchStore(
    (store) => store.contentReviewRailState,
  );
  const clearContentReviewRailState = useWorkbenchStore(
    (store) => store.clearContentReviewRailState,
  );
  const themeSkillsRailState = useWorkbenchStore(
    (store) => store.themeSkillsRailState,
  );
  const clearThemeSkillsRailState = useWorkbenchStore(
    (store) => store.clearThemeSkillsRailState,
  );
  const triggerSkill = useWorkbenchStore((store) => store.triggerSkill);

  useEffect(() => {
    if (!shouldRender || !isCreateWorkspaceView) {
      clearContentReviewRailState();
      clearThemeSkillsRailState();
    }
  }, [
    clearContentReviewRailState,
    clearThemeSkillsRailState,
    isCreateWorkspaceView,
    shouldRender,
  ]);

  useEffect(() => {
    if (contentReviewRailState) {
      setCollapsed(false);
    }
  }, [contentReviewRailState]);

  useEffect(() => {
    if (themeSkillsRailState) {
      setCollapsed(false);
    }
  }, [themeSkillsRailState]);

  if (!shouldRender) {
    return null;
  }

  if (contentReviewRailState) {
    return <ContentReviewPanel open={true} {...contentReviewRailState} />;
  }

  if (themeSkillsRailState) {
    return (
      <ThemeWorkbenchSkillsPanel
        skills={themeSkillsRailState.skills}
        currentGate={{
          key: "idle",
          title: "就绪",
          status: themeSkillsRailState.isAutoRunning ? "running" : "idle",
          description: themeSkillsRailState.isAutoRunning
            ? "AI 正在执行任务..."
            : "选择技能开始创作",
        }}
        disabled={themeSkillsRailState.isAutoRunning}
        onTriggerSkill={(skill) => triggerSkill(skill.key)}
        onRequestCollapse={() => clearThemeSkillsRailState()}
      />
    );
  }

  if (!isCreateWorkspaceView) {
    return <NonCreateRail onBackToCreateView={onBackToCreateView} />;
  }

  return collapsed ? (
    <CollapsedRail
      onExpand={() => {
        setPendingExpandedActionKey(null);
        setCollapsed(false);
      }}
      onExpandToAction={(actionKey) => {
        setPendingExpandedActionKey(actionKey);
        setCollapsed(false);
      }}
    />
  ) : (
    <CapabilityPanel
      onCollapse={() => setCollapsed(true)}
      projectId={projectId}
      onCreateContentFromPrompt={onCreateContentFromPrompt}
      initialExpandedActionKey={pendingExpandedActionKey}
      onInitialExpandedActionConsumed={() => setPendingExpandedActionKey(null)}
    />
  );
}

export default WorkbenchRightRail;
