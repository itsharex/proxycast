export type SearchResourceType = "image" | "audio" | "bgm";
export type ImageModelType = "basic" | "jimeng" | "kling";
export type ImageSizeType = "16-9" | "9-16" | "1-1";
export type CoverPlatformType = "bilibili" | "xiaohongshu" | "douyin";
export type CoverCountType = "1" | "2" | "3";
export type VideoAssetModelType = "keling" | "jimeng" | "wan-2-5";
export type VideoAssetVersionType = "v2-1-master" | "v2" | "v1-6";
export type VideoAssetRatioType = "16-9" | "9-16" | "1-1";
export type VideoAssetDurationType = "5s";

export const SEARCH_RESOURCE_OPTIONS: Array<{
  value: SearchResourceType;
  label: string;
}> = [
  { value: "audio", label: "音效" },
  { value: "bgm", label: "背景音乐" },
  { value: "image", label: "图片" },
];

export const IMAGE_MODEL_OPTIONS: Array<{
  value: ImageModelType;
  label: string;
  disabled?: boolean;
}> = [
  { value: "basic", label: "基础模型" },
  { value: "jimeng", label: "即梦" },
  { value: "kling", label: "可灵", disabled: true },
];

export const IMAGE_SIZE_OPTIONS: Array<{
  value: ImageSizeType;
  label: string;
}> = [
  { value: "16-9", label: "16:9 横图" },
  { value: "9-16", label: "9:16 竖图" },
  { value: "1-1", label: "1:1 方图" },
];

export const COVER_PLATFORM_OPTIONS: Array<{
  value: CoverPlatformType;
  label: string;
}> = [
  { value: "xiaohongshu", label: "小红书" },
  { value: "douyin", label: "抖音" },
  { value: "bilibili", label: "B站" },
];

export const COVER_COUNT_OPTIONS: Array<{
  value: CoverCountType;
  label: string;
}> = [
  { value: "3", label: "3 张" },
  { value: "2", label: "2 张" },
  { value: "1", label: "1 张" },
];

export const VIDEO_ASSET_MODEL_OPTIONS: Array<{
  value: VideoAssetModelType;
  label: string;
  disabled?: boolean;
}> = [
  { value: "keling", label: "可灵" },
  { value: "jimeng", label: "即梦" },
  { value: "wan-2-5", label: "WAN 2.5", disabled: true },
];

export const VIDEO_ASSET_VERSION_OPTIONS: Array<{
  value: VideoAssetVersionType;
  label: string;
}> = [
  { value: "v2-1-master", label: "2.1 Master" },
  { value: "v2", label: "2.0" },
  { value: "v1-6", label: "1.6" },
];

export const VIDEO_ASSET_RATIO_OPTIONS: Array<{
  value: VideoAssetRatioType;
  label: string;
}> = [
  { value: "1-1", label: "1:1" },
  { value: "9-16", label: "9:16" },
  { value: "16-9", label: "16:9" },
];

export const VIDEO_ASSET_DURATION_OPTIONS: Array<{
  value: VideoAssetDurationType;
  label: string;
}> = [{ value: "5s", label: "5s" }];

export function parseVideoDuration(duration: VideoAssetDurationType): number {
  const value = Number.parseInt(duration.replace("s", ""), 10);
  return Number.isFinite(value) ? value : 5;
}

export function mapImageSizeTypeToResolution(size: ImageSizeType): string {
  if (size === "16-9") {
    return "1792x1024";
  }
  if (size === "9-16") {
    return "1024x1792";
  }
  return "1024x1024";
}

export function mapImageSizeTypeToAspectRatio(size: ImageSizeType): string {
  if (size === "16-9") {
    return "16:9";
  }
  if (size === "9-16") {
    return "9:16";
  }
  return "1:1";
}

export function mapCoverPlatformToResolution(platform: CoverPlatformType): string {
  if (platform === "bilibili") {
    return "1792x1024";
  }
  if (platform === "douyin") {
    return "1024x1792";
  }
  return "1024x1792";
}

export function mapCoverPlatformToAspectRatio(
  platform: CoverPlatformType,
): string {
  if (platform === "bilibili") {
    return "16:9";
  }
  return "9:16";
}
