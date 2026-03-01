/**
 * 剧本画布组件
 *
 * 用于短剧项目的剧本编辑
 */

import React, { memo, useCallback, useEffect } from "react";
import styled from "styled-components";
import { X, Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "sonner";
import type { ScriptCanvasState, Scene, Dialogue } from "./types";
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

const Header = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid hsl(var(--border));
`;

const Title = styled.h3`
  font-size: 14px;
  font-weight: 600;
`;

const Content = styled.div`
  display: flex;
  flex: 1;
  min-height: 0;
`;

const SceneList = styled.div`
  width: 200px;
  border-right: 1px solid hsl(var(--border));
  display: flex;
  flex-direction: column;
`;

const SceneListHeader = styled.div`
  padding: 12px;
  border-bottom: 1px solid hsl(var(--border));
  display: flex;
  align-items: center;
  justify-content: space-between;
`;

const SceneItem = styled.div<{ $active?: boolean; $flash?: boolean }>`
  padding: 12px;
  cursor: pointer;
  border-bottom: 1px solid hsl(var(--border));
  background: ${({ $active }) =>
    $active ? "hsl(var(--accent))" : "transparent"};
  animation: ${({ $flash }) => ($flash ? "scriptInsertFlash 1.6s ease" : "none")};

  &:hover {
    background: hsl(var(--accent));
  }

  @keyframes scriptInsertFlash {
    0% {
      box-shadow: inset 0 0 0 2px hsl(var(--primary) / 0.65);
    }
    100% {
      box-shadow: inset 0 0 0 0 hsl(var(--primary) / 0);
    }
  }
`;

const SceneNumber = styled.div`
  font-weight: 600;
  font-size: 13px;
`;

const SceneLocation = styled.div`
  font-size: 12px;
  color: hsl(var(--muted-foreground));
`;

const EditorArea = styled.div`
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
`;

const SceneHeader = styled.div`
  padding: 16px;
  border-bottom: 1px solid hsl(var(--border));
  display: flex;
  gap: 12px;
  align-items: center;
`;

const DialogueList = styled.div`
  flex: 1;
  padding: 16px;
`;

const DialogueItem = styled.div`
  margin-bottom: 16px;
  padding: 12px;
  background: hsl(var(--card));
  border-radius: 8px;
  border: 1px solid hsl(var(--border));
`;

const DialogueHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
`;

const _CharacterName = styled.div`
  font-weight: 600;
  font-size: 14px;
  color: hsl(var(--primary));
`;

interface ScriptCanvasProps {
  state: ScriptCanvasState;
  onStateChange: (state: ScriptCanvasState) => void;
  projectId?: string | null;
  contentId?: string | null;
  onBackHome?: () => void;
  onClose: () => void;
}

function appendImageMarkdown(text: string, image: InsertableImage): string {
  const imageUrl = image.contentUrl?.trim();
  if (!imageUrl) return text;
  if (text.includes(`](${imageUrl})`) || text.includes(imageUrl)) {
    return text;
  }
  const alt = image.title?.trim() || "场景参考图";
  const snippet = `![${alt}](${imageUrl})`;
  const trimmed = text.trimEnd();
  return trimmed ? `${trimmed}\n\n${snippet}` : snippet;
}

export const ScriptCanvas: React.FC<ScriptCanvasProps> = memo(
  ({ state, onStateChange, projectId, contentId, onBackHome, onClose }) => {
    const [highlightedSceneId, setHighlightedSceneId] = React.useState<
      string | null
    >(null);
    const currentScene = state.scenes.find(
      (s) => s.id === state.currentSceneId,
    );

    const handleSceneSelect = useCallback(
      (sceneId: string) => {
        onStateChange({ ...state, currentSceneId: sceneId });
      },
      [state, onStateChange],
    );

    const handleAddScene = useCallback(() => {
      const newScene: Scene = {
        id: crypto.randomUUID(),
        number: state.scenes.length + 1,
        location: "内景",
        time: "日",
        dialogues: [],
      };
      onStateChange({
        ...state,
        scenes: [...state.scenes, newScene],
        currentSceneId: newScene.id,
      });
    }, [state, onStateChange]);

    const handleUpdateScene = useCallback(
      (updates: Partial<Scene>) => {
        if (!currentScene) return;
        const updatedScenes = state.scenes.map((s) =>
          s.id === currentScene.id ? { ...s, ...updates } : s,
        );
        onStateChange({ ...state, scenes: updatedScenes });
      },
      [state, currentScene, onStateChange],
    );

    const handleAddDialogue = useCallback(() => {
      if (!currentScene) return;
      const newDialogue: Dialogue = {
        id: crypto.randomUUID(),
        characterId: "",
        characterName: "角色",
        content: "",
      };
      handleUpdateScene({
        dialogues: [...currentScene.dialogues, newDialogue],
      });
    }, [currentScene, handleUpdateScene]);

    const handleUpdateDialogue = useCallback(
      (dialogueId: string, updates: Partial<Dialogue>) => {
        if (!currentScene) return;
        const updatedDialogues = currentScene.dialogues.map((d) =>
          d.id === dialogueId ? { ...d, ...updates } : d,
        );
        handleUpdateScene({ dialogues: updatedDialogues });
      },
      [currentScene, handleUpdateScene],
    );

    const handleDeleteDialogue = useCallback(
      (dialogueId: string) => {
        if (!currentScene) return;
        const updatedDialogues = currentScene.dialogues.filter(
          (d) => d.id !== dialogueId,
        );
        handleUpdateScene({ dialogues: updatedDialogues });
      },
      [currentScene, handleUpdateScene],
    );

    const matchesRequestTarget = useCallback(
      (request: CanvasImageInsertRequest): boolean =>
        matchesCanvasImageInsertTarget(request, {
          projectId: projectId || null,
          contentId: contentId || null,
          canvasType: "script",
        }),
      [contentId, projectId],
    );

    const processInsertRequest = useCallback(
      (request: CanvasImageInsertRequest) => {
        if (!matchesRequestTarget(request)) {
          return;
        }

        const nowScene =
          state.scenes.find((scene) => scene.id === state.currentSceneId) ||
          state.scenes[0] ||
          null;
        if (!nowScene) {
          emitCanvasImageInsertAck({
            requestId: request.requestId,
            success: false,
            canvasType: "script",
            reason: "no_scene",
          });
          ackCanvasImageInsertRequest(request.requestId);
          return;
        }

        const nextDescription = appendImageMarkdown(
          nowScene.description || "",
          request.image,
        );
        if (nextDescription === (nowScene.description || "")) {
          emitCanvasImageInsertAck({
            requestId: request.requestId,
            success: false,
            canvasType: "script",
            locationLabel: `第${nowScene.number}场`,
            reason: "duplicate",
          });
          ackCanvasImageInsertRequest(request.requestId);
          return;
        }

        const updatedScenes = state.scenes.map((scene) =>
          scene.id === nowScene.id
            ? {
                ...scene,
                description: nextDescription,
              }
            : scene,
        );
        onStateChange({
          ...state,
          scenes: updatedScenes,
          currentSceneId: nowScene.id,
        });
        setHighlightedSceneId(nowScene.id);
        toast.success(`已插入到剧本第${nowScene.number}场`);

        emitCanvasImageInsertAck({
          requestId: request.requestId,
          success: true,
          canvasType: "script",
          locationLabel: `第${nowScene.number}场 场景描述`,
        });
        ackCanvasImageInsertRequest(request.requestId);
      },
      [matchesRequestTarget, onStateChange, state],
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
      if (!highlightedSceneId) {
        return;
      }
      const timer = window.setTimeout(() => {
        setHighlightedSceneId((prev) =>
          prev === highlightedSceneId ? null : prev,
        );
      }, 1600);
      return () => window.clearTimeout(timer);
    }, [highlightedSceneId]);

    return (
      <Container>
        <CanvasBreadcrumbHeader label="剧本" onBackHome={onBackHome} />

        <InnerContainer>
          <Header>
            <Title>剧本</Title>
            <Button variant="ghost" size="icon" onClick={onClose}>
              <X className="h-4 w-4" />
            </Button>
          </Header>

          <Content>
            <SceneList>
              <SceneListHeader>
                <span className="text-sm font-medium">场景</span>
                <Button variant="ghost" size="icon" onClick={handleAddScene}>
                  <Plus className="h-4 w-4" />
                </Button>
              </SceneListHeader>
              <ScrollArea className="flex-1">
                {state.scenes.map((scene) => (
                  <SceneItem
                    key={scene.id}
                    $active={scene.id === state.currentSceneId}
                    $flash={scene.id === highlightedSceneId}
                    onClick={() => handleSceneSelect(scene.id)}
                  >
                    <SceneNumber>第{scene.number}场</SceneNumber>
                    <SceneLocation>
                      {scene.location}（{scene.time}）
                    </SceneLocation>
                  </SceneItem>
                ))}
              </ScrollArea>
            </SceneList>

            <EditorArea>
              {currentScene && (
                <>
                  <SceneHeader>
                    <Input
                      value={currentScene.location}
                      onChange={(e) =>
                        handleUpdateScene({ location: e.target.value })
                      }
                      placeholder="场景地点"
                      className="w-40"
                    />
                    <Input
                      value={currentScene.time}
                      onChange={(e) =>
                        handleUpdateScene({ time: e.target.value })
                      }
                      placeholder="时间"
                      className="w-20"
                    />
                    <Textarea
                      value={currentScene.description || ""}
                      onChange={(e) =>
                        handleUpdateScene({ description: e.target.value })
                      }
                      placeholder="场景描述..."
                      className="flex-1 min-h-[40px] resize-none"
                    />
                  </SceneHeader>

                  <ScrollArea className="flex-1">
                    <DialogueList>
                      {currentScene.dialogues.map((dialogue) => (
                        <DialogueItem key={dialogue.id}>
                          <DialogueHeader>
                            <Input
                              value={dialogue.characterName}
                              onChange={(e) =>
                                handleUpdateDialogue(dialogue.id, {
                                  characterName: e.target.value,
                                })
                              }
                              placeholder="角色名"
                              className="w-32"
                            />
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => handleDeleteDialogue(dialogue.id)}
                            >
                              <Trash2 className="h-4 w-4" />
                            </Button>
                          </DialogueHeader>
                          <Textarea
                            value={dialogue.content}
                            onChange={(e) =>
                              handleUpdateDialogue(dialogue.id, {
                                content: e.target.value,
                              })
                            }
                            placeholder="对白内容..."
                            className="min-h-[60px]"
                          />
                        </DialogueItem>
                      ))}
                      <Button
                        variant="outline"
                        className="w-full"
                        onClick={handleAddDialogue}
                      >
                        <Plus className="h-4 w-4 mr-2" />
                        添加对白
                      </Button>
                    </DialogueList>
                  </ScrollArea>
                </>
              )}
            </EditorArea>
          </Content>
        </InnerContainer>
      </Container>
    );
  },
);

ScriptCanvas.displayName = "ScriptCanvas";
