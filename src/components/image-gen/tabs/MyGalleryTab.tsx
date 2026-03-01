/**
 * @file 我的图片库 Tab
 * @description 显示用户已保存的图片素材库
 * @module components/image-gen/tabs/MyGalleryTab
 */

import { ImageGallery } from "@/components/content-creator/material/ImageGallery";
import type { PosterMaterial } from "@/types/poster-material";
import { convertFileSrc } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Images } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import styled from "styled-components";
import { getActiveContentTarget } from "@/lib/activeContentTarget";
import {
  emitCanvasImageInsertRequest,
  onCanvasImageInsertAck,
  type CanvasImageInsertAck,
  type CanvasImageTargetType,
} from "@/lib/canvasImageInsertBus";
import {
  addCanvasImageInsertHistory,
  getCanvasImageInsertHistory,
  type CanvasImageInsertHistoryEntry,
} from "@/lib/canvasImageInsertHistory";
import type { Page, PageParams } from "@/types/page";

export interface MyGalleryTabProps {
  /** 项目 ID */
  projectId?: string | null;
  /** 页面跳转 */
  onNavigate?: (page: Page, params?: PageParams) => void;
}

const Container = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px 20px;
`;

const ActionBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding: 10px 12px;
  border: 1px solid hsl(var(--border) / 0.6);
  border-radius: 10px;
  background: hsl(var(--card) / 0.5);
`;

const ActionHint = styled.div`
  font-size: 12px;
  color: hsl(var(--muted-foreground));
`;

const InsertButton = styled.button`
  border: 1px solid hsl(var(--border));
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  border-radius: 8px;
  font-size: 12px;
  padding: 6px 12px;
  cursor: pointer;
  transition: all 0.2s ease;

  &:hover:not(:disabled) {
    border-color: hsl(var(--primary) / 0.5);
    background: hsl(var(--accent) / 0.5);
  }

  &:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
`;

const RelocateButton = styled(InsertButton)`
  font-size: 11px;
  padding: 5px 10px;
`;

const EmptyState = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: hsl(var(--muted-foreground));
  text-align: center;
  padding: 48px;
`;

const EmptyIcon = styled.div`
  width: 64px;
  height: 64px;
  border-radius: 20px;
  background: hsl(var(--muted) / 0.2);
  display: flex;
  align-items: center;
  justify-content: center;
  color: hsl(var(--muted-foreground) / 0.5);
`;

const EmptyTitle = styled.p`
  margin: 0;
  font-size: 15px;
  font-weight: 500;
  color: hsl(var(--foreground) / 0.7);
`;

const EmptyHint = styled.p`
  margin: 0;
  font-size: 13px;
`;

function normalizeCanvasType(
  value: string | null | undefined,
): CanvasImageTargetType {
  if (
    value === "document" ||
    value === "novel" ||
    value === "script" ||
    value === "music" ||
    value === "poster" ||
    value === "video"
  ) {
    return value;
  }
  return "document";
}

function mapCanvasTypeToTheme(canvasType: CanvasImageTargetType): string {
  switch (canvasType) {
    case "poster":
      return "poster";
    case "music":
      return "music";
    case "novel":
      return "novel";
    case "video":
      return "video";
    case "script":
      return "social-media";
    case "document":
    case "auto":
    default:
      return "document";
  }
}

function getVisibleInsertHistory(
  projectId?: string | null,
): CanvasImageInsertHistoryEntry[] {
  const history = getCanvasImageInsertHistory();
  const filtered = projectId
    ? history.filter((entry) => entry.projectId === projectId)
    : history;
  return filtered.slice(0, 3);
}

export function MyGalleryTab({ projectId, onNavigate }: MyGalleryTabProps) {
  const [selectedMaterial, setSelectedMaterial] = useState<PosterMaterial | null>(
    null,
  );
  const [recentInsertHistory, setRecentInsertHistory] = useState<
    CanvasImageInsertHistoryEntry[]
  >(() => getVisibleInsertHistory(projectId));
  const pendingInsertRequestMetaRef = useRef<
    Map<
      string,
      {
        projectId: string;
        contentId: string | null;
        canvasType: CanvasImageTargetType;
        theme: string;
        imageTitle?: string;
      }
    >
  >(new Map());

  useEffect(() => {
    setRecentInsertHistory(getVisibleInsertHistory(projectId));
  }, [projectId]);

  useEffect(() => {
    const unsubscribe = onCanvasImageInsertAck((ack: CanvasImageInsertAck) => {
      const pendingMeta = pendingInsertRequestMetaRef.current.get(ack.requestId);
      if (!pendingMeta) {
        return;
      }
      pendingInsertRequestMetaRef.current.delete(ack.requestId);

      if (!ack.success) {
        toast.error("插图失败，请返回创作区重试");
        return;
      }

      const nextHistory = addCanvasImageInsertHistory({
        requestId: ack.requestId,
        projectId: pendingMeta.projectId,
        contentId: pendingMeta.contentId,
        canvasType: pendingMeta.canvasType,
        theme: pendingMeta.theme,
        imageTitle: pendingMeta.imageTitle,
        locationLabel: ack.locationLabel,
      });
      setRecentInsertHistory(
        (projectId
          ? nextHistory.filter((entry) => entry.projectId === projectId)
          : nextHistory
        ).slice(0, 3),
      );
    });

    return unsubscribe;
  }, [projectId]);

  const handleInsertFromGallery = (material: PosterMaterial) => {
    if (!projectId) {
      toast.error("请先选择项目");
      return;
    }

    const imageUrl = material.filePath
      ? convertFileSrc(material.filePath)
      : material.metadata?.thumbnail || "";
    if (!imageUrl) {
      toast.error("该素材缺少可用图片地址，无法插入");
      return;
    }

    const target = getActiveContentTarget();
    const sameProjectTarget = target?.projectId === projectId ? target : null;
    const targetContentId = sameProjectTarget?.contentId ?? null;
    const targetCanvasType = normalizeCanvasType(sameProjectTarget?.canvasType);
    const targetTheme = mapCanvasTypeToTheme(targetCanvasType);

    const request = emitCanvasImageInsertRequest({
      projectId,
      contentId: targetContentId,
      canvasType: targetCanvasType,
      anchorHint:
        targetCanvasType === "video"
          ? "video_start_frame"
          : targetCanvasType === "poster"
            ? "poster_center"
            : "section_end",
      source: "gallery",
      image: {
        id: material.id,
        previewUrl: material.metadata?.thumbnail || imageUrl,
        contentUrl: imageUrl,
        title: material.name,
        width: material.metadata?.width,
        height: material.metadata?.height,
        attributionName: "项目素材库",
        provider: "gallery",
      },
    });
    pendingInsertRequestMetaRef.current.set(request.requestId, {
      projectId,
      contentId: targetContentId,
      canvasType: targetCanvasType,
      theme: targetTheme,
      imageTitle: material.name,
    });

    onNavigate?.("agent", {
      projectId,
      contentId: targetContentId ?? undefined,
      theme: targetTheme,
      lockTheme: false,
    });
    toast.success("已发送到当前画布，正在自动定位");
  };

  const handleRelocate = (entry: CanvasImageInsertHistoryEntry) => {
    onNavigate?.("agent", {
      projectId: entry.projectId,
      contentId: entry.contentId ?? undefined,
      theme: entry.theme,
      lockTheme: false,
    });
    toast.success("正在定位到插图位置");
  };

  if (!projectId) {
    return (
      <Container>
        <EmptyState>
          <EmptyIcon>
            <Images size={28} />
          </EmptyIcon>
          <EmptyTitle>请先选择项目</EmptyTitle>
          <EmptyHint>在右上角选择一个项目后即可查看图片库</EmptyHint>
        </EmptyState>
      </Container>
    );
  }

  return (
    <Container>
      <ActionBar>
        <ActionHint>
          {selectedMaterial
            ? `已选中：${selectedMaterial.name}`
            : "可双击图片直接插入当前画布，或先单击选中后点击右侧按钮"}
        </ActionHint>
        <div className="flex items-center gap-2">
          {recentInsertHistory[0] && (
            <RelocateButton
              type="button"
              onClick={() => handleRelocate(recentInsertHistory[0])}
            >
              再次定位
            </RelocateButton>
          )}
          <InsertButton
            type="button"
            disabled={!selectedMaterial}
            onClick={() => {
              if (!selectedMaterial) {
                return;
              }
              handleInsertFromGallery(selectedMaterial);
            }}
          >
            插入选中图片到当前画布
          </InsertButton>
        </div>
      </ActionBar>
      <ImageGallery
        projectId={projectId}
        maxHeight="calc(100vh - 200px)"
        selectedIds={selectedMaterial ? [selectedMaterial.id] : []}
        onSelect={(materials) => {
          setSelectedMaterial(materials[0] || null);
        }}
        onDoubleClick={handleInsertFromGallery}
      />
    </Container>
  );
}

export default MyGalleryTab;
