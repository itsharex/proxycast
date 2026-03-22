import { useState } from "react";
import { ChevronDown, FileSearch } from "lucide-react";

export interface GeneratedOutputItem {
  id: string;
  title: string;
  detail: string;
  assetType?: "image" | "audio";
  assetUrl?: string;
  actionLabel?: string;
  onAction?: () => void;
}

export function GeneratedOutputsPanel({
  items,
}: {
  items: GeneratedOutputItem[];
}) {
  const [historyExpanded, setHistoryExpanded] = useState(false);

  if (items.length === 0) {
    return (
      <div className="mt-auto flex min-h-[160px] flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-border/80 bg-muted/20 px-4 py-6 text-center text-xs text-muted-foreground">
        <FileSearch className="h-6 w-6 opacity-50" />
        <p>生成的素材输出将保存在此处。</p>
      </div>
    );
  }

  const latestItem = items[0] ?? null;
  const historyItems = items.slice(1);
  const shouldRenderHistory = historyItems.length > 0;

  return (
    <div className="mt-auto rounded-2xl border border-border/70 bg-muted/15 p-3">
      <div className="flex items-center justify-between gap-3">
        <div>
          <div className="text-xs font-semibold text-foreground/90">任务摘要</div>
          <div className="mt-1 text-[11px] text-muted-foreground">
            主区结果优先，右侧只保留最近任务的调度线索。
          </div>
        </div>
        {shouldRenderHistory ? (
          <button
            type="button"
            data-testid="workbench-generated-output-history-toggle"
            className="inline-flex shrink-0 items-center gap-1 rounded-lg border border-border/60 bg-background/80 px-2.5 py-1 text-[11px] font-medium text-foreground/70 transition-colors hover:border-border hover:text-foreground"
            onClick={() => setHistoryExpanded((current) => !current)}
          >
            <span>{historyExpanded ? "收起历史" : `更早 ${historyItems.length} 条`}</span>
            <ChevronDown
              className={`h-3.5 w-3.5 transition-transform ${
                historyExpanded ? "rotate-180" : ""
              }`}
            />
          </button>
        ) : null}
      </div>

      {latestItem ? (
        <div className="mt-3">
          <div
            key={latestItem.id}
            data-testid="workbench-generated-output-item"
            className="rounded-xl border border-border/60 bg-background/90 p-2.5 shadow-sm"
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="text-[10px] font-semibold uppercase tracking-[0.12em] text-muted-foreground/70">
                  最近任务
                </div>
                <div className="mt-1 text-[12px] font-semibold text-foreground">
                  {latestItem.title}
                </div>
                <div className="mt-1 text-[11px] leading-relaxed text-muted-foreground">
                  {latestItem.detail}
                </div>
              </div>
              {latestItem.actionLabel && latestItem.onAction ? (
                <button
                  type="button"
                  data-testid="workbench-generated-output-action"
                  className="shrink-0 rounded-lg border border-border/70 bg-background px-2.5 py-1 text-[11px] font-medium text-foreground/80 transition-colors hover:border-border hover:text-foreground"
                  onClick={latestItem.onAction}
                >
                  {latestItem.actionLabel}
                </button>
              ) : null}
            </div>
            {latestItem.assetType === "image" && latestItem.assetUrl ? (
              <img
                src={latestItem.assetUrl}
                alt={latestItem.title}
                className="mt-2 h-24 w-full rounded-md object-cover"
                loading="lazy"
                data-testid="workbench-generated-output-image"
              />
            ) : null}
            {latestItem.assetType === "audio" && latestItem.assetUrl ? (
              <audio
                controls
                src={latestItem.assetUrl}
                className="mt-2 w-full"
                data-testid="workbench-generated-output-audio"
              />
            ) : null}
          </div>
        </div>
      ) : null}

      {shouldRenderHistory && historyExpanded ? (
        <div
          className="mt-3 max-h-[220px] space-y-2 overflow-y-auto pr-1"
          data-testid="workbench-generated-output-history"
        >
          {historyItems.map((item) => (
            <div
              key={item.id}
              className="rounded-xl border border-border/40 bg-background/65 px-3 py-2"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="text-[11px] font-medium text-foreground/85">
                    {item.title}
                  </div>
                  <div className="mt-1 text-[10px] leading-relaxed text-muted-foreground">
                    {item.detail}
                  </div>
                </div>
                {item.actionLabel && item.onAction ? (
                  <button
                    type="button"
                    className="shrink-0 rounded-md px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground transition-colors hover:text-foreground"
                    onClick={item.onAction}
                  >
                    {item.actionLabel}
                  </button>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
