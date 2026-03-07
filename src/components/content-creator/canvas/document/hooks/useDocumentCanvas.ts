/**
 * @file 文档画布状态管理 Hook
 * @description 管理文档画布的状态和操作
 * @module components/content-creator/canvas/document/hooks/useDocumentCanvas
 */

import { useState, useCallback, useMemo, useEffect } from "react";
import type {
  DocumentCanvasState,
  DocumentVersion,
  PlatformType,
  ExportFormat,
} from "../types";
import { createInitialDocumentState } from "../types";
import { exportDocumentContent } from "../utils/exportDocument";

/**
 * 文档画布状态管理 Hook
 */
export function useDocumentCanvas(initialContent: string = "") {
  const [state, setState] = useState<DocumentCanvasState>(() =>
    createInitialDocumentState(initialContent),
  );

  // 编辑中的内容（未保存）
  const [editingContent, setEditingContent] = useState<string>(initialContent);

  useEffect(() => {
    if (state.isEditing) {
      setEditingContent(state.content);
    }
  }, [state.content, state.currentVersionId, state.isEditing]);

  /**
   * 当前版本
   */
  const currentVersion = useMemo(() => {
    return state.versions.find((v) => v.id === state.currentVersionId) || null;
  }, [state.versions, state.currentVersionId]);

  /**
   * 更新内容并创建新版本
   */
  const updateContent = useCallback(
    (content: string, description?: string) => {
      const newVersion: DocumentVersion = {
        id: crypto.randomUUID(),
        content,
        createdAt: Date.now(),
        description: description || `版本 ${state.versions.length + 1}`,
      };

      setState((prev) => ({
        ...prev,
        content,
        versions: [...prev.versions, newVersion],
        currentVersionId: newVersion.id,
      }));
    },
    [state.versions.length],
  );

  /**
   * 切换平台
   */
  const switchPlatform = useCallback((platform: PlatformType) => {
    setState((prev) => ({
      ...prev,
      platform,
    }));
  }, []);

  /**
   * 切换版本
   */
  const switchVersion = useCallback(
    (versionId: string) => {
      const version = state.versions.find((v) => v.id === versionId);
      if (version) {
        if (state.isEditing) {
          setEditingContent(version.content);
        }
        setState((prev) => ({
          ...prev,
          content: version.content,
          currentVersionId: versionId,
        }));
      }
    },
    [state.isEditing, state.versions],
  );

  /**
   * 进入编辑模式
   */
  const startEditing = useCallback(() => {
    setEditingContent(state.content);
    setState((prev) => ({
      ...prev,
      isEditing: true,
    }));
  }, [state.content]);

  /**
   * 切换到预览模式（不保存）
   */
  const cancelEditing = useCallback(() => {
    setState((prev) => ({
      ...prev,
      isEditing: false,
    }));
  }, []);

  /**
   * 保存编辑
   */
  const saveEditing = useCallback(() => {
    if (editingContent !== state.content) {
      updateContent(editingContent, "手动编辑");
    }
    setState((prev) => ({
      ...prev,
      isEditing: true,
    }));
    setEditingContent(editingContent);
  }, [editingContent, state.content, updateContent]);

  /**
   * 更新编辑中的内容
   */
  const updateEditingContent = useCallback((content: string) => {
    setEditingContent(content);
  }, []);

  /**
   * 导出文档
   */
  const exportDocument = useCallback(
    async (format: ExportFormat) => {
      await exportDocumentContent(state.content, format);
    },
    [state.content],
  );

  /**
   * 设置完整状态（用于外部控制）
   */
  const setFullState = useCallback((newState: DocumentCanvasState) => {
    setState(newState);
  }, []);

  return {
    state,
    currentVersion,
    editingContent,
    // 操作方法
    updateContent,
    switchPlatform,
    switchVersion,
    startEditing,
    cancelEditing,
    saveEditing,
    updateEditingContent,
    exportDocument,
    setFullState,
  };
}
