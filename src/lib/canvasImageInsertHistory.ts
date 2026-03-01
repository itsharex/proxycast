import type { CanvasImageTargetType } from "@/lib/canvasImageInsertBus";

const INSERT_HISTORY_STORAGE_KEY = "proxycast:canvas-image-insert-history";
const MAX_HISTORY_SIZE = 30;

export interface CanvasImageInsertHistoryEntry {
  requestId: string;
  createdAt: number;
  projectId: string;
  contentId: string | null;
  canvasType: CanvasImageTargetType;
  theme: string;
  imageTitle?: string;
  locationLabel?: string;
}

const hasWindow = () => typeof window !== "undefined";

function normalizeId(value: string | null | undefined): string | null {
  if (!value) return null;
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function getCanvasImageInsertHistory(): CanvasImageInsertHistoryEntry[] {
  if (!hasWindow()) return [];
  try {
    const raw = localStorage.getItem(INSERT_HISTORY_STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((item): item is CanvasImageInsertHistoryEntry => {
      return (
        item &&
        typeof item === "object" &&
        typeof item.requestId === "string" &&
        typeof item.createdAt === "number" &&
        typeof item.projectId === "string" &&
        typeof item.canvasType === "string" &&
        typeof item.theme === "string"
      );
    });
  } catch {
    return [];
  }
}

function writeHistory(history: CanvasImageInsertHistoryEntry[]): void {
  if (!hasWindow()) return;
  localStorage.setItem(
    INSERT_HISTORY_STORAGE_KEY,
    JSON.stringify(history.slice(0, MAX_HISTORY_SIZE)),
  );
}

export function addCanvasImageInsertHistory(
  entry: Omit<CanvasImageInsertHistoryEntry, "createdAt"> & {
    createdAt?: number;
  },
): CanvasImageInsertHistoryEntry[] {
  const normalizedProjectId = normalizeId(entry.projectId);
  if (!normalizedProjectId) {
    return getCanvasImageInsertHistory();
  }

  const history = getCanvasImageInsertHistory();
  const payload: CanvasImageInsertHistoryEntry = {
    requestId: entry.requestId,
    createdAt: entry.createdAt ?? Date.now(),
    projectId: normalizedProjectId,
    contentId: normalizeId(entry.contentId),
    canvasType: entry.canvasType,
    theme: entry.theme,
    imageTitle: entry.imageTitle,
    locationLabel: entry.locationLabel,
  };

  const nextHistory = [
    payload,
    ...history.filter((item) => item.requestId !== payload.requestId),
  ];
  writeHistory(nextHistory);
  return nextHistory;
}

export function clearCanvasImageInsertHistory(): void {
  if (!hasWindow()) return;
  localStorage.removeItem(INSERT_HISTORY_STORAGE_KEY);
}
