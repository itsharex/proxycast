export type ImageWorkbenchCommandMode = "generate" | "edit" | "variation";

export interface ParsedImageWorkbenchCommand {
  rawText: string;
  trigger: "@配图" | "@image" | "/image";
  body: string;
  mode: ImageWorkbenchCommandMode;
  prompt: string;
  count: number;
  size?: string;
  aspectRatio?: string;
  targetRef?: string;
}

const IMAGE_COMMAND_PREFIX_REGEX =
  /^\s*(@配图|@image|\/image)(?:\s+|$)([\s\S]*)$/i;
const TARGET_REF_REGEX = /#(img-[a-z0-9_-]+)/i;
const SIZE_REGEX = /\b(\d{3,4}x\d{3,4})\b/i;
const ASPECT_RATIO_REGEX =
  /\b(1:1|16:9|9:16|4:3|3:4|3:2|2:3|21:9|4:5|5:4)\b/i;

function normalizeTrigger(value: string): "@配图" | "@image" | "/image" {
  const normalized = value.trim().toLowerCase();
  if (normalized === "@image") {
    return "@image";
  }
  if (normalized === "/image") {
    return "/image";
  }
  return "@配图";
}

function clampCount(value: number | null | undefined): number {
  if (!value || !Number.isFinite(value)) {
    return 1;
  }
  return Math.max(1, Math.min(8, Math.trunc(value)));
}

function extractCount(body: string): number {
  const patterns = [
    /(?:出|生成|要)\s*(\d+)\s*张/i,
    /(\d+)\s*张/i,
    /\bx\s*(\d+)\b/i,
  ];

  for (const pattern of patterns) {
    const matched = body.match(pattern);
    if (matched) {
      return clampCount(Number.parseInt(matched[1] || "", 10));
    }
  }

  return 1;
}

function resolveMode(
  normalizedBody: string,
  targetRef?: string,
): ImageWorkbenchCommandMode {
  if (/^(编辑|edit|修改)(?:\s|$|[:：])/i.test(normalizedBody)) {
    return "edit";
  }
  if (/^(重绘|变体|variation|variant)(?:\s|$|[:：])/i.test(normalizedBody)) {
    return "variation";
  }
  if (/^(生成|create|generate)(?:\s|$|[:：])/i.test(normalizedBody)) {
    return "generate";
  }
  return targetRef ? "variation" : "generate";
}

function stripPromptDecorations(body: string): string {
  return body
    .replace(/^(生成|create|generate)(?:\s|$|[:：])*/i, "")
    .replace(/^(编辑|edit|修改)(?:\s|$|[:：])*/i, "")
    .replace(/^(重绘|变体|variation|variant)(?:\s|$|[:：])*/i, "")
    .replace(TARGET_REF_REGEX, "")
    .replace(/(?:出|生成|要)\s*\d+\s*张/gi, "")
    .replace(/\d+\s*张/gi, "")
    .replace(/\bx\s*\d+\b/gi, "")
    .replace(SIZE_REGEX, "")
    .replace(ASPECT_RATIO_REGEX, "")
    .replace(/^[,\s，。；;:：]+|[,\s，。；;:：]+$/g, "")
    .replace(/\s+/g, " ")
    .trim();
}

function resolveSize(body: string): { size?: string; aspectRatio?: string } {
  const explicitSize = body.match(SIZE_REGEX)?.[1];
  if (explicitSize) {
    return { size: explicitSize };
  }

  const aspectRatio = body.match(ASPECT_RATIO_REGEX)?.[1];
  if (!aspectRatio) {
    return {};
  }

  const mappedSizes: Record<string, string> = {
    "1:1": "1024x1024",
    "16:9": "1792x1024",
    "21:9": "1792x1024",
    "4:3": "1152x864",
    "3:2": "1344x768",
    "5:4": "1152x864",
    "9:16": "1024x1792",
    "3:4": "864x1152",
    "2:3": "768x1344",
    "4:5": "864x1152",
  };

  return {
    size: mappedSizes[aspectRatio],
    aspectRatio,
  };
}

export function parseImageWorkbenchCommand(
  text: string,
): ParsedImageWorkbenchCommand | null {
  const matched = text.match(IMAGE_COMMAND_PREFIX_REGEX);
  if (!matched) {
    return null;
  }

  const trigger = normalizeTrigger(matched[1] || "");
  const body = (matched[2] || "").trim();
  const targetRef = body.match(TARGET_REF_REGEX)?.[1];
  const normalizedBody = body.trim();
  const mode = resolveMode(normalizedBody, targetRef);
  const { size, aspectRatio } = resolveSize(normalizedBody);

  return {
    rawText: text,
    trigger,
    body,
    mode,
    prompt: stripPromptDecorations(normalizedBody),
    count: extractCount(normalizedBody),
    size,
    aspectRatio,
    targetRef,
  };
}
