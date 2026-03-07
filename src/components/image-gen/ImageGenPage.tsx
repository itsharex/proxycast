/**
 * @file 图片生成页面
 * @description 插图功能 - 包含图片搜索、AI生图、本地图片、我的图片库四个 Tab
 * @module components/image-gen/ImageGenPage
 */

import { useState, useEffect, useMemo } from "react";
import styled, { keyframes } from "styled-components";
import { useProjects } from "@/hooks/useProjects";
import {
  getStoredResourceProjectId,
  onResourceProjectChange,
  setStoredResourceProjectId,
} from "@/lib/resourceProjectSelection";
import { buildHomeAgentParams } from "@/lib/workspace/navigation";
import type { Page, PageParams } from "@/types/page";
import { ChevronDown } from "lucide-react";
import { CanvasBreadcrumbHeader } from "@/components/content-creator/canvas/shared/CanvasBreadcrumbHeader";

type PageNavigate = (page: Page, params?: PageParams) => void;
import { AiImageGenTab } from "./tabs/AiImageGenTab";
import { ImageSearchTab } from "./tabs/ImageSearchTab";
import { LocalImageTab } from "./tabs/LocalImageTab";
import { MyGalleryTab } from "./tabs/MyGalleryTab";

interface ImageGenPageProps {
  onNavigate?: PageNavigate;
}

// ==================== Styled Components ====================

const PageLayout = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  background: hsl(var(--background));
`;

const HeaderBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 20px 6px;
  background: hsl(var(--background));
  border-bottom: 1px solid hsl(var(--border) / 0.5);
`;

const ProjectSelectorWrapper = styled.div`
  position: relative;
  display: inline-flex;
  align-items: center;
`;

const ProjectSelector = styled.select`
  appearance: none;
  padding: 6px 32px 6px 14px;
  border: 1px solid hsl(var(--border));
  border-radius: 10px;
  background: hsl(var(--card) / 0.6);
  font-size: 13px;
  color: hsl(var(--foreground));
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(8px);

  &:hover {
    border-color: hsl(var(--primary) / 0.5);
    background: hsl(var(--card) / 0.9);
  }

  &:focus {
    outline: none;
    border-color: hsl(var(--primary));
    box-shadow: 0 0 0 3px hsl(var(--primary) / 0.1);
  }
`;

const SelectorIcon = styled.div`
  position: absolute;
  right: 10px;
  pointer-events: none;
  color: hsl(var(--muted-foreground));
`;

const MainContainer = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: hsl(var(--background));
`;

const TabsBar = styled.div`
  display: flex;
  align-items: center;
  gap: 0;
  padding: 0 20px;
  background: hsl(var(--background));
  border-bottom: 1px solid hsl(var(--border) / 0.4);
`;

const slideIn = keyframes`
  from {
    transform: scaleX(0);
  }
  to {
    transform: scaleX(1);
  }
`;

const TabButton = styled.button<{ $active: boolean }>`
  position: relative;
  padding: 12px 24px;
  border: none;
  background: transparent;
  font-size: 14px;
  font-weight: ${({ $active }) => ($active ? 600 : 400)};
  color: ${({ $active }) =>
    $active ? "hsl(var(--primary))" : "hsl(var(--muted-foreground))"};
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;

  &:hover {
    color: ${({ $active }) =>
    $active ? "hsl(var(--primary))" : "hsl(var(--foreground))"};
  }

  &::after {
    content: "";
    position: absolute;
    bottom: -1px;
    left: 12px;
    right: 12px;
    height: 2px;
    border-radius: 2px 2px 0 0;
    background: ${({ $active }) =>
    $active ? "hsl(var(--primary))" : "transparent"};
    transition: background 0.2s ease;
    animation: ${({ $active }) => ($active ? slideIn : "none")} 0.25s ease;
  }
`;

const TabContent = styled.div`
  flex: 1;
  min-height: 0;
  overflow: hidden;
`;

// ==================== Component ====================

const TABS = [
  { key: "ai-gen", label: "AI生图" },
  { key: "search", label: "图片搜索" },
  { key: "local", label: "本地图片" },
  { key: "gallery", label: "我的图片库" },
] as const;

export function ImageGenPage({ onNavigate }: ImageGenPageProps) {
  const [activeTab, setActiveTab] = useState("ai-gen");
  const [selectedProjectId, setSelectedProjectId] = useState<string | null>(null);

  const { projects, defaultProject, loading: projectsLoading } = useProjects();

  const handleBackHome = () => {
    onNavigate?.("agent", buildHomeAgentParams());
  };

  // 可用项目列表（排除已归档）
  const availableProjects = useMemo(
    () => projects.filter((project) => !project.isArchived),
    [projects],
  );

  // 初始化项目 ID
  useEffect(() => {
    if (projectsLoading) {
      return;
    }

    setSelectedProjectId((current) => {
      // 如果当前值有效，保持不变
      if (current && availableProjects.some((project) => project.id === current)) {
        return current;
      }

      // 从存储中获取
      const storedProjectId = getStoredResourceProjectId({ includeLegacy: true });
      if (
        storedProjectId &&
        availableProjects.some((project) => project.id === storedProjectId)
      ) {
        return storedProjectId;
      }

      // 使用默认项目
      const preferredProject =
        (defaultProject && !defaultProject.isArchived ? defaultProject : null) ??
        availableProjects[0];

      return preferredProject?.id || null;
    });
  }, [projectsLoading, availableProjects, defaultProject]);

  // 同步项目 ID 到存储
  useEffect(() => {
    if (selectedProjectId) {
      setStoredResourceProjectId(selectedProjectId, {
        source: "image-gen-target",
        syncLegacy: true,
        emitEvent: true,
      });
    }
  }, [selectedProjectId]);

  // 监听项目变更事件
  useEffect(() => {
    return onResourceProjectChange((detail) => {
      if (detail.source !== "resources") {
        return;
      }

      const nextProjectId = detail.projectId;
      if (!nextProjectId || nextProjectId === selectedProjectId) {
        return;
      }

      if (!availableProjects.some((project) => project.id === nextProjectId)) {
        return;
      }

      setSelectedProjectId(nextProjectId);
    });
  }, [availableProjects, selectedProjectId]);

  return (
    <PageLayout>
      <HeaderBar>
        <CanvasBreadcrumbHeader label="插图" onBackHome={handleBackHome} />
        <ProjectSelectorWrapper>
          <ProjectSelector
            value={selectedProjectId || ""}
            onChange={(e) => setSelectedProjectId(e.target.value || null)}
            disabled={projectsLoading}
          >
            <option value="">选择项目</option>
            {availableProjects.map((project) => (
              <option key={project.id} value={project.id}>
                {project.name}
              </option>
            ))}
          </ProjectSelector>
          <SelectorIcon>
            <ChevronDown size={14} />
          </SelectorIcon>
        </ProjectSelectorWrapper>
      </HeaderBar>

      <MainContainer>
        <TabsBar>
          {TABS.map((tab) => (
            <TabButton
              key={tab.key}
              $active={activeTab === tab.key}
              onClick={() => setActiveTab(tab.key)}
            >
              {tab.label}
            </TabButton>
          ))}
        </TabsBar>

        <TabContent>
          {activeTab === "search" && (
            <ImageSearchTab
              projectId={selectedProjectId}
              onNavigate={onNavigate}
            />
          )}
          {activeTab === "ai-gen" && (
            <AiImageGenTab projectId={selectedProjectId} onNavigate={onNavigate} />
          )}
          {activeTab === "local" && (
            <LocalImageTab projectId={selectedProjectId} />
          )}
          {activeTab === "gallery" && (
            <MyGalleryTab
              projectId={selectedProjectId}
              onNavigate={onNavigate}
            />
          )}
        </TabContent>
      </MainContainer>
    </PageLayout>
  );
}

export default ImageGenPage;
