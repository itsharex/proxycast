/**
 * 内容同步 Hook
 *
 * 提供防抖同步、状态管理和失败重试功能
 */

import { useState, useCallback, useRef } from "react";
import { updateContent } from "@/lib/api/project";

export type SyncStatus = "idle" | "syncing" | "success" | "error";

interface UseContentSyncOptions {
  /** 防抖延迟（毫秒） */
  debounceMs?: number;
  /** 是否自动重试 */
  autoRetry?: boolean;
  /** 重试延迟（毫秒） */
  retryDelayMs?: number;
}

interface UseContentSyncReturn {
  /** 同步内容 */
  syncContent: (contentId: string, body: string) => void;
  /** 同步状态 */
  syncStatus: SyncStatus;
  /** 手动重置状态 */
  resetStatus: () => void;
}

export function useContentSync(
  options: UseContentSyncOptions = {},
): UseContentSyncReturn {
  const { debounceMs = 2000, autoRetry = true, retryDelayMs = 5000 } = options;

  const [syncStatus, setSyncStatus] = useState<SyncStatus>("idle");
  const syncTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const retryTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const lastSyncDataRef = useRef<{ contentId: string; body: string } | null>(
    null,
  );

  const syncContent = useCallback(
    (contentId: string, body: string) => {
      // 保存最后的同步数据（用于重试）
      lastSyncDataRef.current = { contentId, body };

      // 清除之前的定时器
      if (syncTimeoutRef.current) {
        clearTimeout(syncTimeoutRef.current);
      }
      if (retryTimeoutRef.current) {
        clearTimeout(retryTimeoutRef.current);
      }

      // 防抖：延迟后同步
      syncTimeoutRef.current = setTimeout(async () => {
        setSyncStatus("syncing");

        try {
          await updateContent(contentId, { body });
          setSyncStatus("success");

          // 3 秒后重置状态
          setTimeout(() => {
            setSyncStatus((current) =>
              current === "success" ? "idle" : current,
            );
          }, 3000);
        } catch (error) {
          console.error("同步内容失败:", error);
          setSyncStatus("error");

          // 自动重试
          if (autoRetry && lastSyncDataRef.current) {
            retryTimeoutRef.current = setTimeout(() => {
              if (lastSyncDataRef.current) {
                console.log("[useContentSync] 重试同步...");
                syncContent(
                  lastSyncDataRef.current.contentId,
                  lastSyncDataRef.current.body,
                );
              }
            }, retryDelayMs);
          }
        }
      }, debounceMs);
    },
    [debounceMs, autoRetry, retryDelayMs],
  );

  const resetStatus = useCallback(() => {
    setSyncStatus("idle");
    if (syncTimeoutRef.current) {
      clearTimeout(syncTimeoutRef.current);
    }
    if (retryTimeoutRef.current) {
      clearTimeout(retryTimeoutRef.current);
    }
  }, []);

  return { syncContent, syncStatus, resetStatus };
}
