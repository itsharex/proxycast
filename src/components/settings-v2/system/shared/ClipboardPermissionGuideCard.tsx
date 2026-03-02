import { useMemo, useState } from "react";
import { AlertTriangle, ExternalLink } from "lucide-react";
import { open as openExternal } from "@tauri-apps/plugin-shell";
import { cn } from "@/lib/utils";
import { getClipboardPermissionGuide } from "@/lib/crashDiagnostic";

interface ClipboardPermissionGuideCardProps {
  className?: string;
}

export function ClipboardPermissionGuideCard({
  className,
}: ClipboardPermissionGuideCardProps) {
  const [openError, setOpenError] = useState<string | null>(null);

  const guide = useMemo(
    () => getClipboardPermissionGuide(navigator.platform, navigator.userAgent),
    [],
  );

  const handleOpenSettings = async () => {
    if (!guide.settingsUrl) return;
    setOpenError(null);
    try {
      await openExternal(guide.settingsUrl);
    } catch (error) {
      try {
        window.open(guide.settingsUrl, "_blank");
      } catch {
        setOpenError(
          error instanceof Error ? error.message : "打开系统设置失败",
        );
      }
    }
  };

  return (
    <div
      className={cn(
        "rounded-lg border border-amber-200 bg-amber-50/80 p-3 text-sm dark:border-amber-900/40 dark:bg-amber-950/20",
        className,
      )}
    >
      <div className="flex items-start gap-2">
        <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0 text-amber-600 dark:text-amber-400" />
        <div className="min-w-0 flex-1 space-y-2">
          <p className="font-medium text-amber-900 dark:text-amber-300">
            {guide.title}
          </p>
          <ol className="list-decimal space-y-1 pl-4 text-amber-800 dark:text-amber-400">
            {guide.steps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
          {guide.settingsUrl && (
            <button
              type="button"
              onClick={() => void handleOpenSettings()}
              className="inline-flex items-center gap-1 rounded-md border border-amber-300 px-2.5 py-1 text-xs text-amber-900 transition-colors hover:bg-amber-100 dark:border-amber-700 dark:text-amber-200 dark:hover:bg-amber-900/40"
            >
              打开系统设置
              <ExternalLink className="h-3.5 w-3.5" />
            </button>
          )}
          {openError && (
            <p className="text-xs text-destructive">
              打开系统设置失败：{openError}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
