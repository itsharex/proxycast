import { useEffect } from "react";
import { wechatChannelSetRuntimeModel } from "@/lib/api/channelsRuntime";

let lastSyncedWechatRuntimeModelKey: string | null = null;

interface UseWechatRuntimeModelSyncOptions {
  providerId?: string | null;
  modelId?: string | null;
  source: string;
}

export function useWechatRuntimeModelSync({
  providerId,
  modelId,
  source,
}: UseWechatRuntimeModelSyncOptions) {
  useEffect(() => {
    const normalizedProviderId = providerId?.trim() || "";
    const normalizedModelId = modelId?.trim() || "";
    const selectionKey =
      normalizedProviderId && normalizedModelId
        ? `${normalizedProviderId}/${normalizedModelId}`
        : null;

    if (!selectionKey) {
      lastSyncedWechatRuntimeModelKey = null;
      return;
    }

    if (lastSyncedWechatRuntimeModelKey === selectionKey) {
      return;
    }

    let cancelled = false;

    void wechatChannelSetRuntimeModel({
      providerId: normalizedProviderId,
      modelId: normalizedModelId,
    })
      .then(() => {
        if (!cancelled) {
          lastSyncedWechatRuntimeModelKey = selectionKey;
        }
      })
      .catch((error) => {
        if (!cancelled) {
          console.warn(
            `[WechatRuntimeModelSync] ${source} 同步微信运行时模型失败:`,
            error,
          );
        }
      });

    return () => {
      cancelled = true;
    };
  }, [modelId, providerId, source]);
}
