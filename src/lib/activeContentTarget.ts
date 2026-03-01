const ACTIVE_CONTENT_TARGET_KEY = "proxycast-active-content-target";

export type ActiveCanvasType =
  | "document"
  | "novel"
  | "script"
  | "music"
  | "poster"
  | "video"
  | null;

export interface ActiveContentTarget {
  projectId: string | null;
  contentId: string | null;
  canvasType?: ActiveCanvasType;
  updatedAt: number;
}

const hasWindow = () => typeof window !== "undefined";

const normalizeId = (value: string | null | undefined): string | null => {
  if (!value) return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
};

const normalizeCanvasType = (
  value: string | null | undefined,
): ActiveCanvasType => {
  if (!value) return null;
  const normalized = value.trim().toLowerCase();
  if (
    normalized === "document" ||
    normalized === "novel" ||
    normalized === "script" ||
    normalized === "music" ||
    normalized === "poster" ||
    normalized === "video"
  ) {
    return normalized;
  }
  return null;
};

export const getActiveContentTarget = (): ActiveContentTarget | null => {
  if (!hasWindow()) return null;
  try {
    const raw = localStorage.getItem(ACTIVE_CONTENT_TARGET_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<ActiveContentTarget>;
    if (!parsed || typeof parsed !== "object") return null;
    return {
      projectId: normalizeId(parsed.projectId),
      contentId: normalizeId(parsed.contentId),
      canvasType: normalizeCanvasType(
        typeof parsed.canvasType === "string" ? parsed.canvasType : null,
      ),
      updatedAt:
        typeof parsed.updatedAt === "number" ? parsed.updatedAt : Date.now(),
    };
  } catch {
    return null;
  }
};

export const setActiveContentTarget = (
  projectId?: string | null,
  contentId?: string | null,
  canvasType?: ActiveCanvasType,
) => {
  if (!hasWindow()) return;
  const payload: ActiveContentTarget = {
    projectId: normalizeId(projectId),
    contentId: normalizeId(contentId),
    canvasType: normalizeCanvasType(canvasType || null),
    updatedAt: Date.now(),
  };

  if (!payload.projectId && !payload.contentId) {
    localStorage.removeItem(ACTIVE_CONTENT_TARGET_KEY);
    return;
  }

  localStorage.setItem(ACTIVE_CONTENT_TARGET_KEY, JSON.stringify(payload));
};
