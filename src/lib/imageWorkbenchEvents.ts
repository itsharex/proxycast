export const IMAGE_WORKBENCH_REQUEST_EVENT = "lime:image-workbench-request";
export const IMAGE_WORKBENCH_FOCUS_EVENT = "lime:image-workbench-focus";

export type ImageWorkbenchExternalRequestSource = "workspace-right-rail";
export type ImageWorkbenchExternalRequestTarget = "generate" | "cover";
export type ImageWorkbenchExternalRequestModelPreset =
  | "basic"
  | "jimeng"
  | "kling";

export interface ImageWorkbenchExternalRequestDetail {
  requestId: string;
  source: ImageWorkbenchExternalRequestSource;
  projectId?: string | null;
  contentId?: string | null;
  prompt: string;
  target: ImageWorkbenchExternalRequestTarget;
  aspectRatio?: string;
  count?: number;
  modelPreset?: ImageWorkbenchExternalRequestModelPreset;
}

export interface ImageWorkbenchFocusDetail {
  source?: ImageWorkbenchExternalRequestSource;
  projectId?: string | null;
  contentId?: string | null;
}

function hasWindow(): boolean {
  return typeof window !== "undefined";
}

export function emitImageWorkbenchRequest(
  input: Omit<ImageWorkbenchExternalRequestDetail, "requestId"> & {
    requestId?: string;
  },
): ImageWorkbenchExternalRequestDetail {
  const detail: ImageWorkbenchExternalRequestDetail = {
    ...input,
    requestId: input.requestId || crypto.randomUUID(),
  };

  if (hasWindow()) {
    window.dispatchEvent(
      new CustomEvent<ImageWorkbenchExternalRequestDetail>(
        IMAGE_WORKBENCH_REQUEST_EVENT,
        { detail },
      ),
    );
  }

  return detail;
}

export function onImageWorkbenchRequest(
  listener: (detail: ImageWorkbenchExternalRequestDetail) => void,
): () => void {
  if (!hasWindow()) {
    return () => undefined;
  }

  const handler = (event: Event) => {
    if (!(event instanceof CustomEvent)) {
      return;
    }

    const detail = (
      event as CustomEvent<ImageWorkbenchExternalRequestDetail>
    ).detail;
    if (!detail || typeof detail.requestId !== "string") {
      return;
    }
    listener(detail);
  };

  window.addEventListener(IMAGE_WORKBENCH_REQUEST_EVENT, handler);
  return () => {
    window.removeEventListener(IMAGE_WORKBENCH_REQUEST_EVENT, handler);
  };
}

export function emitImageWorkbenchFocus(
  detail: ImageWorkbenchFocusDetail,
): void {
  if (!hasWindow()) {
    return;
  }

  window.dispatchEvent(
    new CustomEvent<ImageWorkbenchFocusDetail>(IMAGE_WORKBENCH_FOCUS_EVENT, {
      detail,
    }),
  );
}

export function onImageWorkbenchFocus(
  listener: (detail: ImageWorkbenchFocusDetail) => void,
): () => void {
  if (!hasWindow()) {
    return () => undefined;
  }

  const handler = (event: Event) => {
    if (!(event instanceof CustomEvent)) {
      return;
    }

    const detail = (event as CustomEvent<ImageWorkbenchFocusDetail>).detail;
    listener(detail || {});
  };

  window.addEventListener(IMAGE_WORKBENCH_FOCUS_EVENT, handler);
  return () => {
    window.removeEventListener(IMAGE_WORKBENCH_FOCUS_EVENT, handler);
  };
}
