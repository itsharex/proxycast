/**
 * @file 音乐画布主组件
 * @description 整合工具栏、歌词编辑器、简谱渲染器等
 * @module components/content-creator/canvas/music/MusicCanvas
 */

import React, { memo, useMemo, useCallback, useState, useEffect } from "react";
import styled from "styled-components";
import type { MusicCanvasProps, MusicViewMode } from "./types";
import { createSection } from "./types";
import { MusicToolbar } from "./MusicToolbar";
import {
  NumberedNotationRenderer,
  GuitarTabRenderer,
  PianoRollRenderer,
} from "./renderers";
import { Copy, Check, Music } from "lucide-react";
import {
  ackCanvasImageInsertRequest,
  emitCanvasImageInsertAck,
  getPendingCanvasImageInsertRequests,
  matchesCanvasImageInsertTarget,
  onCanvasImageInsertRequest,
  type CanvasImageInsertRequest,
} from "@/lib/canvasImageInsertBus";

/** 段落类型中文映射 */
const SECTION_DISPLAY_NAMES: Record<string, string> = {
  intro: "前奏",
  verse: "主歌",
  "pre-chorus": "预副歌",
  chorus: "副歌",
  bridge: "桥段",
  interlude: "间奏",
  outro: "尾奏",
};

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

const MainContent = styled.div`
  flex: 1;
  display: flex;
  overflow: hidden;
`;

const EditorPane = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 16px;
`;

const SectionTitle = styled.h3`
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin: 0 0 12px 0;
  display: flex;
  align-items: center;
  gap: 8px;
`;

const LyricsContainer = styled.div`
  flex: 1;
  overflow-y: auto;
  background: hsl(var(--muted) / 0.3);
  border-radius: 8px;
  padding: 16px;
`;

const SectionBlock = styled.div<{ $isSelected: boolean; $flash?: boolean }>`
  margin-bottom: 16px;
  padding: 12px;
  border-radius: 6px;
  background: ${({ $isSelected }) =>
    $isSelected ? "hsl(var(--accent) / 0.1)" : "transparent"};
  border: 1px solid
    ${({ $isSelected }) =>
    $isSelected ? "hsl(var(--primary))" : "hsl(var(--border))"};
  cursor: pointer;
  animation: ${({ $flash }) => ($flash ? "musicInsertFlash 1.6s ease" : "none")};
  transition: all 0.2s;

  &:hover {
    background: hsl(var(--accent) / 0.05);
  }

  @keyframes musicInsertFlash {
    0% {
      box-shadow: 0 0 0 0 hsl(var(--primary) / 0.55);
    }
    100% {
      box-shadow: 0 0 0 0 hsl(var(--primary) / 0);
    }
  }
`;

const SectionHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
`;

const SectionTag = styled.span`
  font-size: 12px;
  font-weight: 600;
  color: hsl(var(--primary));
  background: hsl(var(--primary) / 0.1);
  padding: 2px 8px;
  border-radius: 4px;
`;

const SectionName = styled.span`
  font-size: 13px;
  color: hsl(var(--muted-foreground));
`;

const LyricsLine = styled.p`
  font-size: 16px;
  line-height: 2;
  color: hsl(var(--foreground));
  margin: 6px 0;
  white-space: pre-wrap;
`;

const EmptyStateContainer = styled.div`
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  width: 100%;
  color: hsl(var(--muted-foreground));
  text-align: center;
  padding: 40px;
  animation: fadeIn 0.5s ease-out;

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
  }
`;

const IconWrapper = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 72px;
  height: 72px;
  border-radius: 20px;
  background: hsl(var(--primary) / 0.1);
  color: hsl(var(--primary));
  margin-bottom: 24px;
  box-shadow: 0 4px 20px hsl(var(--primary) / 0.05);
  position: relative;
  
  &::after {
    content: '';
    position: absolute;
    inset: -10px;
    border-radius: 26px;
    background: hsl(var(--primary) / 0.05);
    z-index: -1;
  }
`;

const EmptyTitle = styled.h3`
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: hsl(var(--foreground));
  letter-spacing: -0.01em;
`;

const EmptyDescription = styled.p`
  font-size: 14px;
  line-height: 1.6;
  margin: 0;
  max-width: 320px;
  color: hsl(var(--muted-foreground));
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

const StatusBar = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: hsl(var(--muted) / 0.5);
  border-top: 1px solid hsl(var(--border));
  font-size: 12px;
  color: hsl(var(--muted-foreground));
`;

const StatusItem = styled.span`
  display: flex;
  align-items: center;
  gap: 4px;
`;

/**
 * 音乐画布主组件
 */
export const MusicCanvas: React.FC<MusicCanvasProps> = memo(
  ({
    state,
    onStateChange,
    projectId,
    contentId,
    onClose,
    isStreaming = false,
  }) => {
    const [toastMessage, setToastMessage] = useState("");
    const [showToast, setShowToast] = useState(false);
    const [highlightedSectionId, setHighlightedSectionId] = useState<
      string | null
    >(null);

    // 统计信息
    const stats = useMemo(() => {
      const totalSections = state.sections.length;
      const totalLines = state.sections.reduce(
        (sum, s) => sum + s.lyricsLines.length,
        0,
      );
      const totalChars = state.sections.reduce(
        (sum, s) =>
          sum +
          s.lyricsLines.reduce((lineSum, line) => lineSum + line.length, 0),
        0,
      );
      return { totalSections, totalLines, totalChars };
    }, [state.sections]);

    // 显示提示
    const showMessage = useCallback((message: string) => {
      setToastMessage(message);
      setShowToast(true);
      setTimeout(() => setShowToast(false), 2000);
    }, []);

    const matchesRequestTarget = useCallback(
      (request: CanvasImageInsertRequest): boolean =>
        matchesCanvasImageInsertTarget(request, {
          projectId: projectId || null,
          contentId: contentId || null,
          canvasType: "music",
        }),
      [contentId, projectId],
    );

    const processInsertRequest = useCallback(
      (request: CanvasImageInsertRequest) => {
        if (!matchesRequestTarget(request)) {
          return;
        }

        const imageUrl = request.image.contentUrl?.trim();
        if (!imageUrl) {
          emitCanvasImageInsertAck({
            requestId: request.requestId,
            success: false,
            canvasType: "music",
            reason: "invalid_image_url",
          });
          ackCanvasImageInsertRequest(request.requestId);
          return;
        }

        let sections = [...state.sections];
        let targetSection =
          sections.find((section) => section.id === state.currentSectionId) ||
          sections[0] ||
          null;

        if (!targetSection) {
          targetSection = createSection("verse", 1);
          sections = [targetSection];
        }

        if (
          targetSection.lyricsLines.some(
            (line) => line.includes(`](${imageUrl})`) || line.includes(imageUrl),
          )
        ) {
          emitCanvasImageInsertAck({
            requestId: request.requestId,
            success: false,
            canvasType: "music",
            locationLabel: `${targetSection.name} 段落`,
            reason: "duplicate",
          });
          ackCanvasImageInsertRequest(request.requestId);
          return;
        }

        const alt = request.image.title?.trim() || "灵感配图";
        const imageLine = `![${alt}](${imageUrl})`;
        sections = sections.map((section) =>
          section.id === targetSection?.id
            ? {
                ...section,
                lyricsLines: [...section.lyricsLines, imageLine],
              }
            : section,
        );

        onStateChange({
          ...state,
          sections,
          currentSectionId: targetSection.id,
        });
        setHighlightedSectionId(targetSection.id);
        showMessage(`🖼️ 已插入到音乐段落「${targetSection.name}」`);

        emitCanvasImageInsertAck({
          requestId: request.requestId,
          success: true,
          canvasType: "music",
          locationLabel: `${targetSection.name} 段落末尾`,
        });
        ackCanvasImageInsertRequest(request.requestId);
      },
      [matchesRequestTarget, onStateChange, showMessage, state],
    );

    useEffect(() => {
      const unsubscribe = onCanvasImageInsertRequest((request) => {
        processInsertRequest(request);
      });
      return unsubscribe;
    }, [processInsertRequest]);

    useEffect(() => {
      getPendingCanvasImageInsertRequests().forEach((request) => {
        processInsertRequest(request);
      });
    }, [processInsertRequest]);

    useEffect(() => {
      if (!highlightedSectionId) {
        return;
      }
      const timer = window.setTimeout(() => {
        setHighlightedSectionId((prev) =>
          prev === highlightedSectionId ? null : prev,
        );
      }, 1600);
      return () => window.clearTimeout(timer);
    }, [highlightedSectionId]);

    // 切换视图模式
    const handleViewModeChange = useCallback(
      (viewMode: MusicViewMode) => {
        onStateChange({ ...state, viewMode });
      },
      [state, onStateChange],
    );

    // 播放/暂停
    const handlePlayToggle = useCallback(() => {
      onStateChange({ ...state, isPlaying: !state.isPlaying });
    }, [state, onStateChange]);

    // 选择段落
    const handleSectionSelect = useCallback(
      (sectionId: string) => {
        onStateChange({ ...state, currentSectionId: sectionId });
      },
      [state, onStateChange],
    );

    // 撤销
    const handleUndo = useCallback(() => {
      // TODO: 实现撤销功能
      showMessage("撤销功能开发中");
    }, [showMessage]);

    // 重做
    const handleRedo = useCallback(() => {
      // TODO: 实现重做功能
      showMessage("重做功能开发中");
    }, [showMessage]);

    // 导出
    const handleExport = useCallback(() => {
      // TODO: 打开导出对话框
      showMessage("导出功能开发中");
    }, [showMessage]);

    // 复制歌词
    const [isCopied, setIsCopied] = useState(false);
    const handleCopyLyrics = useCallback(() => {
      const text = state.sections
        .map((section) => {
          const typeName =
            SECTION_DISPLAY_NAMES[section.type] || section.type.toUpperCase();
          const header = `[${typeName}]`;
          const content = section.lyricsLines.join("\n");
          return `${header}\n${content}`;
        })
        .join("\n\n");

      navigator.clipboard.writeText(text).then(() => {
        setIsCopied(true);
        showMessage("歌词已复制到剪贴板");
        setTimeout(() => setIsCopied(false), 2000);
      });
    }, [state.sections, showMessage]);

    // 渲染歌词视图
    const renderLyricsView = () => {
      if (state.sections.length === 0) {
        return (
          <EmptyStateContainer>
            <IconWrapper>
              <Music size={32} />
            </IconWrapper>
            <EmptyTitle>开始创作你的歌曲</EmptyTitle>
            <EmptyDescription>
              在左侧对话中描述你想要创作的歌曲，AI 将帮助你完成歌词创作
            </EmptyDescription>
          </EmptyStateContainer>
        );
      }

      return (
        <LyricsContainer>
          {state.sections.map((section) => (
            <SectionBlock
              key={section.id}
              $isSelected={section.id === state.currentSectionId}
              $flash={section.id === highlightedSectionId}
              onClick={() => handleSectionSelect(section.id)}
            >
              <SectionHeader>
                <SectionTag>
                  [
                  {SECTION_DISPLAY_NAMES[section.type] ||
                    section.type.toUpperCase()}
                  ]
                </SectionTag>
                {section.name !== SECTION_DISPLAY_NAMES[section.type] &&
                  section.name !== section.type &&
                  section.name !==
                  (SECTION_DISPLAY_NAMES[section.type] ||
                    section.type.toUpperCase()) && (
                    <SectionName>{section.name}</SectionName>
                  )}
              </SectionHeader>
              {section.lyricsLines.map((line, index) => (
                <LyricsLine key={index}>{line}</LyricsLine>
              ))}
            </SectionBlock>
          ))}
        </LyricsContainer>
      );
    };

    // 渲染简谱视图
    const renderNumberedView = () => {
      return (
        <NumberedNotationRenderer
          sections={state.sections}
          currentSectionId={state.currentSectionId}
          onSectionSelect={handleSectionSelect}
        />
      );
    };

    // 渲染吉他谱视图
    const renderGuitarView = () => {
      return (
        <GuitarTabRenderer
          sections={state.sections}
          currentSectionId={state.currentSectionId}
          onSectionSelect={handleSectionSelect}
        />
      );
    };

    // 渲染钢琴谱视图
    const renderPianoView = () => {
      return (
        <PianoRollRenderer
          sections={state.sections}
          currentSectionId={state.currentSectionId}
          onSectionSelect={handleSectionSelect}
        />
      );
    };

    // 根据视图模式渲染内容
    const renderContent = () => {
      switch (state.viewMode) {
        case "lyrics":
          return renderLyricsView();
        case "numbered":
          return renderNumberedView();
        case "guitar":
          return renderGuitarView();
        case "piano":
          return renderPianoView();
        default:
          return renderLyricsView();
      }
    };

    return (
      <Container>
        <InnerContainer>
          <MusicToolbar
            spec={state.spec}
            viewMode={state.viewMode}
            isPlaying={state.isPlaying}
            canUndo={false}
            canRedo={false}
            onViewModeChange={handleViewModeChange}
            onPlayToggle={handlePlayToggle}
            onUndo={handleUndo}
            onRedo={handleRedo}
            onExport={handleExport}
            onClose={onClose}
          />

          <ContentArea>
            <MainContent>
              <EditorPane>
                <SectionTitle>
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: "8px",
                    }}
                  >
                    {state.viewMode === "lyrics" && "🎤 歌词"}
                    {state.viewMode === "numbered" && "🎼 简谱"}
                    {state.viewMode === "guitar" && "🎸 吉他谱"}
                    {state.viewMode === "piano" && "🎹 钢琴谱"}
                    {isStreaming && (
                      <span
                        style={{ fontSize: 12, color: "hsl(var(--accent))" }}
                      >
                        生成中...
                      </span>
                    )}
                  </div>
                  {state.viewMode === "lyrics" && (
                    <button
                      onClick={handleCopyLyrics}
                      style={{
                        marginLeft: "auto",
                        background: "none",
                        border: "none",
                        cursor: "pointer",
                        display: "flex",
                        alignItems: "center",
                        gap: "4px",
                        fontSize: "12px",
                        color: isCopied
                          ? "hsl(var(--accent))"
                          : "hsl(var(--muted-foreground))",
                        transition: "color 0.2s",
                      }}
                      title="复制歌词"
                    >
                      {isCopied ? <Check size={14} /> : <Copy size={14} />}
                      {isCopied ? "已复制" : "复制"}
                    </button>
                  )}
                </SectionTitle>
                {renderContent()}
              </EditorPane>
            </MainContent>
          </ContentArea>

          <StatusBar>
            <StatusItem>
              🎵 {state.spec.title} | {state.spec.key} | {state.spec.tempo} BPM
            </StatusItem>
            <StatusItem>
              {stats.totalSections} 段 | {stats.totalLines} 行 |{" "}
              {stats.totalChars} 字
            </StatusItem>
          </StatusBar>

          <Toast $visible={showToast}>{toastMessage}</Toast>
        </InnerContainer>
      </Container>
    );
  },
);

MusicCanvas.displayName = "MusicCanvas";
