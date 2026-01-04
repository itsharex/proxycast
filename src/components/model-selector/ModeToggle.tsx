/**
 * 模式切换组件 - 简单模式/专家模式
 */

import { Settings2, Wand2 } from "lucide-react";
import { cn } from "@/lib/utils";

export type SelectionMode = "simple" | "expert";

interface ModeToggleProps {
  /** 当前模式 */
  mode: SelectionMode;
  /** 模式变化回调 */
  onModeChange: (mode: SelectionMode) => void;
  /** 是否禁用 */
  disabled?: boolean;
  /** 自定义类名 */
  className?: string;
}

export function ModeToggle({
  mode,
  onModeChange,
  disabled = false,
  className,
}: ModeToggleProps) {
  return (
    <div className={cn("flex items-center gap-2", className)}>
      <button
        type="button"
        onClick={() => onModeChange("simple")}
        disabled={disabled}
        className={cn(
          "flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors",
          mode === "simple"
            ? "bg-primary text-primary-foreground"
            : "text-muted-foreground hover:text-foreground hover:bg-muted",
        )}
      >
        <Wand2 className="h-4 w-4" />
        <span>简单</span>
      </button>
      <button
        type="button"
        onClick={() => onModeChange("expert")}
        disabled={disabled}
        className={cn(
          "flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors",
          mode === "expert"
            ? "bg-primary text-primary-foreground"
            : "text-muted-foreground hover:text-foreground hover:bg-muted",
        )}
      >
        <Settings2 className="h-4 w-4" />
        <span>专家</span>
      </button>
    </div>
  );
}
