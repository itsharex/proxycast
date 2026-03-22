import React, { useEffect, useState } from "react";
import { Play, X } from "lucide-react";
import type { QueuedTurnSnapshot } from "@/lib/api/agentRuntime";

interface QueuedTurnsPanelProps {
  queuedTurns: QueuedTurnSnapshot[];
  onPromoteQueuedTurn?: (queuedTurnId: string) => void | Promise<boolean>;
  onRemoveQueuedTurn?: (queuedTurnId: string) => void | Promise<boolean>;
}

export const QueuedTurnsPanel: React.FC<QueuedTurnsPanelProps> = ({
  queuedTurns,
  onPromoteQueuedTurn,
  onRemoveQueuedTurn,
}) => {
  const [expandedTurnId, setExpandedTurnId] = useState<string | null>(null);
  const [pendingAction, setPendingAction] = useState<{
    queuedTurnId: string;
    type: "promote" | "remove";
  } | null>(null);

  useEffect(() => {
    if (
      expandedTurnId &&
      !queuedTurns.some((item) => item.queued_turn_id === expandedTurnId)
    ) {
      setExpandedTurnId(null);
    }
  }, [expandedTurnId, queuedTurns]);

  if (queuedTurns.length === 0) {
    return null;
  }

  const runQueuedAction = async (
    queuedTurnId: string,
    type: "promote" | "remove",
  ) => {
    const handler =
      type === "promote" ? onPromoteQueuedTurn : onRemoveQueuedTurn;
    if (!handler) {
      return;
    }

    setPendingAction({ queuedTurnId, type });
    try {
      await handler(queuedTurnId);
    } finally {
      setPendingAction((current) =>
        current?.queuedTurnId === queuedTurnId && current.type === type
          ? null
          : current,
      );
    }
  };

  return (
    <div className="px-3 pb-2">
      <div className="mb-2 flex items-center justify-between text-[11px] font-medium uppercase tracking-[0.16em] text-muted-foreground">
        <span>稍后处理 {queuedTurns.length}</span>
        <span>会依次开始</span>
      </div>
      <div className="flex flex-col gap-2">
        {queuedTurns.map((item) => {
          const messageText = item.message_text.trim()
            ? item.message_text
            : item.message_preview || "空白输入";
          const title = item.message_preview.trim()
            ? item.message_preview
            : messageText;
          const isExpanded = expandedTurnId === item.queued_turn_id;
          const detailId = `queued-turn-detail-${item.queued_turn_id}`;
          const isPromoting =
            pendingAction?.queuedTurnId === item.queued_turn_id &&
            pendingAction.type === "promote";
          const isRemoving =
            pendingAction?.queuedTurnId === item.queued_turn_id &&
            pendingAction.type === "remove";
          const isBusy = isPromoting || isRemoving;

          return (
            <div
              key={item.queued_turn_id}
              className="flex items-start gap-2 rounded-xl border border-border/80 bg-background/80 px-3 py-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.55)]"
            >
              <button
                type="button"
                className="flex min-w-0 flex-1 items-start gap-2 text-left"
                onClick={() =>
                  setExpandedTurnId((prev) =>
                    prev === item.queued_turn_id ? null : item.queued_turn_id,
                  )
                }
                aria-expanded={isExpanded}
                aria-controls={detailId}
              >
                <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-secondary text-[11px] font-semibold text-secondary-foreground">
                  {item.position}
                </div>
                <div className="min-w-0 flex-1">
                  <div className="flex items-center gap-2">
                    <div className="min-w-0 flex-1 truncate text-sm font-medium text-foreground">
                      {title}
                    </div>
                    <span className="shrink-0 text-[11px] font-medium text-muted-foreground">
                      {isExpanded ? "收起" : "查看"}
                    </span>
                  </div>
                  <div className="mt-0.5 text-xs text-muted-foreground">
                    {item.image_count > 0
                      ? `附图 ${item.image_count} 张`
                      : "纯文本请求"}
                  </div>
                  {isExpanded ? (
                    <div
                      id={detailId}
                      className="mt-2 whitespace-pre-wrap break-words text-xs leading-5 text-foreground/80"
                    >
                      {messageText}
                    </div>
                  ) : null}
                </div>
              </button>
              <div className="flex shrink-0 items-center gap-2">
                <button
                  type="button"
                  className="inline-flex h-8 items-center gap-1.5 rounded-full border border-sky-200/80 bg-sky-50 px-3 text-xs font-medium text-sky-700 transition hover:border-sky-300 hover:bg-sky-100 disabled:cursor-not-allowed disabled:opacity-60"
                  onClick={() =>
                    void runQueuedAction(item.queued_turn_id, "promote")
                  }
                  disabled={isBusy}
                  aria-label="插队立即执行"
                >
                  <Play size={13} />
                  <span>{isPromoting ? "切换中" : "立即执行"}</span>
                </button>
                <button
                  type="button"
                  className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-border/80 text-muted-foreground transition hover:border-destructive/40 hover:bg-destructive/5 hover:text-destructive disabled:cursor-not-allowed disabled:opacity-60"
                  onClick={() =>
                    void runQueuedAction(item.queued_turn_id, "remove")
                  }
                  disabled={isBusy}
                  aria-label="移除排队消息"
                >
                  <X size={14} />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};
