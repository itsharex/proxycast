import {
  ackCanvasImageInsertRequest,
  emitCanvasImageInsertRequest,
  getPendingCanvasImageInsertRequests,
  onCanvasImageInsertRequest,
  type CanvasImageInsertRequest,
  type CanvasImageInsertSource,
  type EmitCanvasImageInsertRequestInput,
  type InsertableImage,
} from "@/lib/canvasImageInsertBus";

export type DocumentImageInsertSource = CanvasImageInsertSource;
export type DocumentImageInsertRequest = CanvasImageInsertRequest;
export type EmitDocumentImageInsertRequestInput = Omit<
  EmitCanvasImageInsertRequestInput,
  "canvasType"
>;
export type { InsertableImage };

export const getPendingDocumentImageInsertRequests = (): DocumentImageInsertRequest[] =>
  getPendingCanvasImageInsertRequests().filter(
    (request) =>
      request.canvasType === "document" || request.canvasType === "auto",
  );

export const ackDocumentImageInsertRequest = (requestId: string): void => {
  ackCanvasImageInsertRequest(requestId);
};

export const emitDocumentImageInsertRequest = (
  input: EmitDocumentImageInsertRequestInput,
): DocumentImageInsertRequest =>
  emitCanvasImageInsertRequest({
    ...input,
    canvasType: "document",
  });

export const onDocumentImageInsertRequest = (
  listener: (request: DocumentImageInsertRequest) => void,
): (() => void) =>
  onCanvasImageInsertRequest((request) => {
    if (request.canvasType !== "document" && request.canvasType !== "auto") {
      return;
    }
    listener(request);
  });
