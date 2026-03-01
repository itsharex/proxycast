/**
 * @file 文档画布主组件
 * @description 整合工具栏、渲染器、编辑器、平台标签
 * @module components/content-creator/canvas/document/DocumentCanvas
 */

import React, { memo, useMemo, useCallback, useState, useEffect } from "react";
import styled from "styled-components";
import { invoke } from "@tauri-apps/api/core";
import type { DocumentCanvasProps, ExportFormat, PlatformType } from "./types";
import { DocumentToolbar } from "./DocumentToolbar";
import { DocumentRenderer } from "./DocumentRenderer";
import { NotionEditor } from "./editor";
import { PlatformTabs } from "./PlatformTabs";
import { CanvasBreadcrumbHeader } from "../shared/CanvasBreadcrumbHeader";
import {
  ackCanvasImageInsertRequest,
  emitCanvasImageInsertAck,
  getPendingCanvasImageInsertRequests,
  matchesCanvasImageInsertTarget,
  onCanvasImageInsertRequest,
  type CanvasImageInsertRequest,
  type InsertableImage,
} from "@/lib/canvasImageInsertBus";
import {
  applySectionImageAssignments,
  appendImageToMarkdown,
  buildSectionSearchQuery,
  extractLevel2Sections,
} from "./utils/autoImageInsert";

interface WebImageSearchResponse {
  total: number;
  provider: string;
  hits: Array<{
    id: string;
    thumbnail_url?: string;
    content_url?: string;
    width?: number;
    height?: number;
    name?: string;
    host_page_url?: string;
  }>;
}

interface PixabaySearchResponse {
  total: number;
  total_hits?: number;
  hits: Array<{
    id: number;
    preview_url?: string;
    large_image_url?: string;
    image_width?: number;
    image_height?: number;
    tags?: string;
    page_url?: string;
    user?: string;
  }>;
}

const Container = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  width: 100%;
  padding: 16px;
  gap: 8px;
`;

const InnerContainer = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  background: hsl(var(--background));
  border-radius: 12px;
  border: 1px solid hsl(var(--border));
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
`;

const ContentArea = styled.div`
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
`;

const Toast = styled.div<{ $visible: boolean }>`
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  padding: 12px 24px;
  background: hsl(var(--foreground));
  color: hsl(var(--background));
  border-radius: 8px;
  font-size: 14px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  opacity: ${({ $visible }) => ($visible ? 1 : 0)};
  visibility: ${({ $visible }) => ($visible ? "visible" : "hidden")};
  transition: all 0.3s;
  z-index: 1000;
`;

/**
 * 文档画布主组件
 */
export const DocumentCanvas: React.FC<DocumentCanvasProps> = memo(
  ({
    state,
    onStateChange,
    onBackHome,
    onClose,
    isStreaming = false,
    onSelectionTextChange,
    projectId,
    contentId,
    autoImageTopic,
  }) => {
    const [editingContent, setEditingContent] = useState("");
    const [toastMessage, setToastMessage] = useState("");
    const [showToast, setShowToast] = useState(false);
    const [autoInsertLoading, setAutoInsertLoading] = useState(false);
    const [pendingEditorInsert, setPendingEditorInsert] = useState<{
      requestId: string;
      image: InsertableImage;
    } | null>(null);

    // 当前版本
    const currentVersion = useMemo(() => {
      return (
        state.versions.find((v) => v.id === state.currentVersionId) || null
      );
    }, [state.versions, state.currentVersionId]);

    useEffect(() => {
      onSelectionTextChange?.("");
    }, [state.currentVersionId, state.isEditing, onSelectionTextChange]);

    // 显示提示
    const showMessage = useCallback((message: string) => {
      setToastMessage(message);
      setShowToast(true);
      setTimeout(() => setShowToast(false), 2000);
    }, []);

    const appendImageIntoDocument = useCallback(
      (image: InsertableImage, description = "插入图片") => {
        const baseContent = state.content;
        const nextContent = appendImageToMarkdown(baseContent, image, true);
        if (nextContent === baseContent) {
          showMessage("ℹ️ 图片已存在，跳过插入");
          return false;
        }

        const newVersion = {
          id: crypto.randomUUID(),
          content: nextContent,
          createdAt: Date.now(),
          description,
        };
        onStateChange({
          ...state,
          content: nextContent,
          versions: [...state.versions, newVersion],
          currentVersionId: newVersion.id,
        });
        return true;
      },
      [onStateChange, showMessage, state],
    );

    const matchesRequestTarget = useCallback(
      (request: CanvasImageInsertRequest): boolean =>
        matchesCanvasImageInsertTarget(request, {
          projectId: projectId || null,
          contentId: contentId || null,
          canvasType: "document",
        }),
      [contentId, projectId],
    );

    const processInsertRequest = useCallback(
      (request: CanvasImageInsertRequest) => {
        if (!matchesRequestTarget(request)) {
          return;
        }

        if (state.isEditing) {
          setPendingEditorInsert({
            requestId: request.requestId,
            image: request.image,
          });
          return;
        }

        const inserted = appendImageIntoDocument(request.image, "手动插图");
        if (inserted) {
          showMessage("🖼️ 已插入文稿");
        }
        emitCanvasImageInsertAck({
          requestId: request.requestId,
          success: inserted,
          canvasType: "document",
          locationLabel: inserted ? "文档正文末尾" : "文档中已存在同图",
          reason: inserted ? undefined : "duplicate",
        });
        ackCanvasImageInsertRequest(request.requestId);
      },
      [appendImageIntoDocument, matchesRequestTarget, showMessage, state.isEditing],
    );

    useEffect(() => {
      const unsubscribe = onCanvasImageInsertRequest((request) => {
        processInsertRequest(request);
      });

      return unsubscribe;
    }, [processInsertRequest]);

    useEffect(() => {
      const pendingRequests = getPendingCanvasImageInsertRequests();
      pendingRequests.forEach((request) => {
        processInsertRequest(request);
      });
    }, [processInsertRequest]);

    const mapWebHitToInsertable = useCallback(
      (hit: WebImageSearchResponse["hits"][number], provider: string) => {
        const contentUrl = hit.content_url || hit.thumbnail_url || "";
        const previewUrl = hit.thumbnail_url || hit.content_url || "";
        if (!contentUrl || !previewUrl) {
          return null;
        }
        return {
          id: hit.id || crypto.randomUUID(),
          previewUrl,
          contentUrl,
          pageUrl: hit.host_page_url,
          title: hit.name || "插图",
          width: hit.width,
          height: hit.height,
          attributionName: provider || "Pexels",
          provider,
        } as InsertableImage;
      },
      [],
    );

    const mapPixabayHitToInsertable = useCallback(
      (hit: PixabaySearchResponse["hits"][number]) => {
        const contentUrl = hit.large_image_url || hit.preview_url || "";
        const previewUrl = hit.preview_url || hit.large_image_url || "";
        if (!contentUrl || !previewUrl) {
          return null;
        }
        return {
          id: String(hit.id || crypto.randomUUID()),
          previewUrl,
          contentUrl,
          pageUrl: hit.page_url,
          title: hit.tags || "插图",
          width: hit.image_width,
          height: hit.image_height,
          attributionName: "Pixabay",
          provider: "pixabay",
        } as InsertableImage;
      },
      [],
    );

    const searchImageWithFallback = useCallback(
      async (query: string): Promise<InsertableImage | null> => {
        if (!query.trim()) {
          return null;
        }

        try {
          const webResp = await invoke<WebImageSearchResponse>("search_web_images", {
            req: {
              query,
              page: 1,
              perPage: 6,
            },
          });
          const fromWeb = webResp.hits
            .map((hit) => mapWebHitToInsertable(hit, webResp.provider || "pexels"))
            .find(Boolean);
          if (fromWeb) {
            return fromWeb;
          }
        } catch {
          // 回退到 Pixabay
        }

        try {
          const pixabayResp = await invoke<PixabaySearchResponse>(
            "search_pixabay_images",
            {
              req: {
                query,
                page: 1,
                perPage: 6,
              },
            },
          );
          const fromPixabay = pixabayResp.hits
            .map((hit) => mapPixabayHitToInsertable(hit))
            .find(Boolean);
          return fromPixabay || null;
        } catch {
          return null;
        }
      },
      [mapPixabayHitToInsertable, mapWebHitToInsertable],
    );

    const handleAutoInsertImages = useCallback(async () => {
      if (autoInsertLoading) {
        return;
      }
      if (state.isEditing) {
        showMessage("ℹ️ 请先保存当前编辑，再执行主题配图");
        return;
      }

      setAutoInsertLoading(true);
      try {
        const baseContent = state.content;
        const sections = extractLevel2Sections(baseContent).slice(0, 6);
        const sectionTitles =
          sections.length > 0
            ? sections.map((section) => section.title)
            : [autoImageTopic || "文稿主题"];

        const assignments: Array<{
          sectionTitle: string;
          image: InsertableImage;
        }> = [];

        for (let index = 0; index < sectionTitles.length; index += 1) {
          const sectionTitle = sectionTitles[index];
          const query = buildSectionSearchQuery(autoImageTopic, sectionTitle);
          if (!query) {
            continue;
          }
          showMessage(`🖼️ 正在匹配配图 ${index + 1}/${sectionTitles.length}`);
          const image = await searchImageWithFallback(query);
          if (image) {
            assignments.push({
              sectionTitle,
              image,
            });
          }
        }

        if (!assignments.length) {
          showMessage("⚠️ 未找到可用图片，建议手动插图");
          return;
        }

        const nextContent = applySectionImageAssignments(baseContent, assignments, {
          includeAttribution: true,
        });

        if (nextContent === baseContent) {
          showMessage("ℹ️ 当前小节已有图片，未重复插入");
          return;
        }

        const newVersion = {
          id: crypto.randomUUID(),
          content: nextContent,
          createdAt: Date.now(),
          description: "主题自动配图",
        };
        onStateChange({
          ...state,
          content: nextContent,
          versions: [...state.versions, newVersion],
          currentVersionId: newVersion.id,
        });
        showMessage(`✅ 自动配图完成，已插入 ${assignments.length} 张`);
      } finally {
        setAutoInsertLoading(false);
      }
    }, [
      autoImageTopic,
      autoInsertLoading,
      onStateChange,
      searchImageWithFallback,
      showMessage,
      state,
    ]);

    // 切换版本
    const handleVersionChange = useCallback(
      (versionId: string) => {
        const version = state.versions.find((v) => v.id === versionId);
        if (version) {
          onStateChange({
            ...state,
            content: version.content,
            currentVersionId: versionId,
          });
        }
      },
      [state, onStateChange],
    );

    // 进入编辑模式
    const handleEditToggle = useCallback(() => {
      setEditingContent(state.content);
      onStateChange({ ...state, isEditing: true });
    }, [state, onStateChange]);

    // 保存编辑
    const handleSave = useCallback(() => {
      if (editingContent !== state.content) {
        const newVersion = {
          id: crypto.randomUUID(),
          content: editingContent,
          createdAt: Date.now(),
          description: "手动编辑",
        };
        onStateChange({
          ...state,
          content: editingContent,
          versions: [...state.versions, newVersion],
          currentVersionId: newVersion.id,
          isEditing: false,
        });
        showMessage("✅ 保存成功");
      } else {
        onStateChange({ ...state, isEditing: false });
      }
      setEditingContent("");
    }, [editingContent, state, onStateChange, showMessage]);

    // 取消编辑
    const handleCancel = useCallback(() => {
      setEditingContent("");
      onStateChange({ ...state, isEditing: false });
    }, [state, onStateChange]);

    // 导出文档
    const handleExport = useCallback(
      async (format: ExportFormat) => {
        const content = state.content;

        switch (format) {
          case "markdown": {
            const blob = new Blob([content], { type: "text/markdown" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "document.md";
            a.click();
            URL.revokeObjectURL(url);
            showMessage("📄 已导出 Markdown 文件");
            break;
          }
          case "text": {
            const plainText = content
              .replace(/#{1,6}\s/g, "")
              .replace(/\*\*(.+?)\*\*/g, "$1")
              .replace(/\*(.+?)\*/g, "$1")
              .replace(/`(.+?)`/g, "$1")
              .replace(/\[(.+?)\]\(.+?\)/g, "$1");
            const blob = new Blob([plainText], { type: "text/plain" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = "document.txt";
            a.click();
            URL.revokeObjectURL(url);
            showMessage("📝 已导出纯文本文件");
            break;
          }
          case "clipboard": {
            await navigator.clipboard.writeText(content);
            showMessage("📋 已复制到剪贴板");
            break;
          }
        }
      },
      [state.content, showMessage],
    );

    // 切换平台
    const handlePlatformChange = useCallback(
      (platform: PlatformType) => {
        onStateChange({ ...state, platform });
      },
      [state, onStateChange],
    );

    return (
      <Container>
        <CanvasBreadcrumbHeader label="文档" onBackHome={onBackHome} />

        <InnerContainer>
          <DocumentToolbar
            currentVersion={currentVersion}
            versions={state.versions}
            isEditing={state.isEditing}
            onVersionChange={handleVersionChange}
            onEditToggle={handleEditToggle}
            onSave={handleSave}
            onCancel={handleCancel}
            onExport={handleExport}
            onAutoInsertImages={handleAutoInsertImages}
            autoInsertLoading={autoInsertLoading}
            onClose={onClose}
          />

          <ContentArea>
            {state.isEditing ? (
              <NotionEditor
                content={editingContent}
                onChange={setEditingContent}
                onSave={handleSave}
                onCancel={handleCancel}
                onSelectionTextChange={onSelectionTextChange}
                externalImageInsert={
                  pendingEditorInsert
                    ? {
                        requestId: pendingEditorInsert.requestId,
                        url: pendingEditorInsert.image.contentUrl,
                        alt: pendingEditorInsert.image.title || "插图",
                      }
                    : null
                }
                onExternalImageInsertComplete={(requestId, success) => {
                  if (success) {
                    showMessage("🖼️ 已插入文稿（编辑态）");
                  } else {
                    showMessage("⚠️ 插图失败，请重试");
                  }
                  emitCanvasImageInsertAck({
                    requestId,
                    success,
                    canvasType: "document",
                    locationLabel: success ? "文档编辑器当前光标位置" : undefined,
                    reason: success ? undefined : "editor_insert_failed",
                  });
                  ackCanvasImageInsertRequest(requestId);
                  setPendingEditorInsert((prev) =>
                    prev?.requestId === requestId ? null : prev,
                  );
                }}
              />
            ) : (
              <DocumentRenderer
                content={state.content}
                platform={state.platform}
                isStreaming={isStreaming}
                onSelectionTextChange={onSelectionTextChange}
              />
            )}
          </ContentArea>

          {!state.isEditing && (
            <PlatformTabs
              currentPlatform={state.platform}
              onPlatformChange={handlePlatformChange}
            />
          )}
        </InnerContainer>

        <Toast $visible={showToast}>{toastMessage}</Toast>
      </Container>
    );
  },
);

DocumentCanvas.displayName = "DocumentCanvas";
