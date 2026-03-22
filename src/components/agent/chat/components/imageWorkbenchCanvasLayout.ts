import type { ImageWorkbenchViewport } from "./ImageWorkbenchCanvas";

export interface ImageWorkbenchCanvasTaskMetric {
  expectedCount: number;
  outputCount: number;
  hasFailureMessage?: boolean;
  expanded?: boolean;
}

export interface ImageWorkbenchCanvasLayout {
  columns: number;
  cardWidth: number;
  boardWidth: number;
  boardHeight: number;
  surfaceWidth: number;
  surfaceHeight: number;
}

interface ResolveImageWorkbenchCanvasLayoutParams {
  tasks: ImageWorkbenchCanvasTaskMetric[];
  containerWidth: number;
  containerHeight: number;
}

interface ResolveImageWorkbenchFitViewportParams {
  containerWidth: number;
  containerHeight: number;
  boardWidth: number;
  boardHeight: number;
}

const IMAGE_WORKBENCH_CARD_GAP = 24;
const IMAGE_WORKBENCH_CANVAS_PADDING_X = 32;
const IMAGE_WORKBENCH_CANVAS_PADDING_Y = 44;
const IMAGE_WORKBENCH_TASK_CARD_MIN_WIDTH = 440;
const IMAGE_WORKBENCH_TASK_CARD_MAX_WIDTH = 700;
const IMAGE_WORKBENCH_TASK_CARD_PREFERRED_WIDTH = 500;
const IMAGE_WORKBENCH_EMPTY_CARD_WIDTH = 640;
const IMAGE_WORKBENCH_EMPTY_CARD_HEIGHT = 380;
const IMAGE_WORKBENCH_TASK_HEADER_HEIGHT = 180;
const IMAGE_WORKBENCH_TASK_SINGLE_TILE_HEIGHT = 232;
const IMAGE_WORKBENCH_TASK_GRID_ROW_HEIGHT = 200;
const IMAGE_WORKBENCH_TASK_FAILURE_HEIGHT = 54;
const IMAGE_WORKBENCH_TASK_EXPANDED_DETAIL_HEIGHT = 560;
const IMAGE_WORKBENCH_TASK_GRID_COLUMNS = 2;
const IMAGE_WORKBENCH_FIT_MARGIN = 24;
const IMAGE_WORKBENCH_MIN_SCALE = 0.55;
const IMAGE_WORKBENCH_MAX_SCALE = 1.9;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function estimateImageWorkbenchTaskCardHeight(
  task: ImageWorkbenchCanvasTaskMetric,
): number {
  const imageSlots = Math.max(1, task.outputCount, task.expectedCount);
  const imageColumns =
    imageSlots === 1 ? 1 : IMAGE_WORKBENCH_TASK_GRID_COLUMNS;
  const rows = Math.max(
    1,
    Math.ceil(imageSlots / imageColumns),
  );
  const gridRowHeight =
    imageColumns === 1
      ? IMAGE_WORKBENCH_TASK_SINGLE_TILE_HEIGHT
      : IMAGE_WORKBENCH_TASK_GRID_ROW_HEIGHT;
  return (
    IMAGE_WORKBENCH_TASK_HEADER_HEIGHT +
    rows * gridRowHeight +
    (rows - 1) * 12 +
    (task.hasFailureMessage ? IMAGE_WORKBENCH_TASK_FAILURE_HEIGHT : 0) +
    (task.expanded ? IMAGE_WORKBENCH_TASK_EXPANDED_DETAIL_HEIGHT : 0)
  );
}

export function resolveImageWorkbenchCanvasLayout({
  tasks,
  containerWidth,
  containerHeight,
}: ResolveImageWorkbenchCanvasLayoutParams): ImageWorkbenchCanvasLayout {
  const taskCount = Math.max(tasks.length, 1);
  const safeContainerWidth = Math.max(containerWidth, 960);
  const safeContainerHeight = Math.max(containerHeight, 720);
  const usableWidth = Math.max(
    safeContainerWidth - IMAGE_WORKBENCH_CANVAS_PADDING_X * 2,
    IMAGE_WORKBENCH_TASK_CARD_MIN_WIDTH,
  );
  const maxColumns = Math.max(
    1,
    Math.floor(
      (usableWidth + IMAGE_WORKBENCH_CARD_GAP) /
        (IMAGE_WORKBENCH_TASK_CARD_MIN_WIDTH + IMAGE_WORKBENCH_CARD_GAP),
    ),
  );
  const preferredColumns = Math.max(
    1,
    Math.floor(
      (usableWidth + IMAGE_WORKBENCH_CARD_GAP) /
        (IMAGE_WORKBENCH_TASK_CARD_PREFERRED_WIDTH + IMAGE_WORKBENCH_CARD_GAP),
    ),
  );
  const columns = Math.min(taskCount, Math.max(1, Math.min(maxColumns, preferredColumns)));
  const computedCardWidth =
    (usableWidth - IMAGE_WORKBENCH_CARD_GAP * (columns - 1)) / columns;
  const cardWidth = clamp(
    computedCardWidth,
    IMAGE_WORKBENCH_TASK_CARD_MIN_WIDTH,
    IMAGE_WORKBENCH_TASK_CARD_MAX_WIDTH,
  );
  const rows = Math.max(1, Math.ceil(taskCount / columns));

  if (tasks.length === 0) {
    const boardWidth = IMAGE_WORKBENCH_EMPTY_CARD_WIDTH;
    const boardHeight = IMAGE_WORKBENCH_EMPTY_CARD_HEIGHT;
    return {
      columns: 1,
      cardWidth: IMAGE_WORKBENCH_EMPTY_CARD_WIDTH,
      boardWidth,
      boardHeight,
      surfaceWidth: Math.max(
        boardWidth + IMAGE_WORKBENCH_CANVAS_PADDING_X * 2,
        safeContainerWidth,
      ),
      surfaceHeight: Math.max(
        boardHeight + IMAGE_WORKBENCH_CANVAS_PADDING_Y * 2,
        safeContainerHeight,
      ),
    };
  }

  const rowHeights = Array.from({ length: rows }, (_, rowIndex) => {
    const rowTasks = tasks.slice(rowIndex * columns, (rowIndex + 1) * columns);
    return Math.max(...rowTasks.map(estimateImageWorkbenchTaskCardHeight));
  });
  const boardWidth = columns * cardWidth + (columns - 1) * IMAGE_WORKBENCH_CARD_GAP;
  const boardHeight =
    rowHeights.reduce((total, height) => total + height, 0) +
    IMAGE_WORKBENCH_CARD_GAP * Math.max(rows - 1, 0);

  return {
    columns,
    cardWidth,
    boardWidth,
    boardHeight,
    surfaceWidth: Math.max(
      boardWidth + IMAGE_WORKBENCH_CANVAS_PADDING_X * 2,
      safeContainerWidth,
    ),
    surfaceHeight: Math.max(
      boardHeight + IMAGE_WORKBENCH_CANVAS_PADDING_Y * 2,
      safeContainerHeight,
    ),
  };
}

export function resolveImageWorkbenchFitViewport({
  containerWidth,
  containerHeight,
  boardWidth,
  boardHeight,
}: ResolveImageWorkbenchFitViewportParams): ImageWorkbenchViewport {
  const safeBoardWidth = Math.max(boardWidth, 1);
  const safeBoardHeight = Math.max(boardHeight, 1);
  const safeContainerWidth = Math.max(containerWidth - IMAGE_WORKBENCH_FIT_MARGIN * 2, 1);
  const safeContainerHeight = Math.max(
    containerHeight - IMAGE_WORKBENCH_FIT_MARGIN * 2,
    1,
  );
  const scale = clamp(
    Math.min(
      safeContainerWidth / safeBoardWidth,
      safeContainerHeight / safeBoardHeight,
    ),
    IMAGE_WORKBENCH_MIN_SCALE,
    IMAGE_WORKBENCH_MAX_SCALE,
  );

  return {
    x: (containerWidth - safeBoardWidth * scale) / 2,
    y: (containerHeight - safeBoardHeight * scale) / 2,
    scale,
  };
}
