import type { LucideIcon } from "lucide-react";

export { WorkbenchLeftSidebar } from "./WorkbenchLeftSidebar";
export type { WorkbenchLeftSidebarProps } from "./WorkbenchLeftSidebar";
export { WorkbenchRightRail } from "./WorkbenchRightRail";
export type { WorkbenchRightRailProps } from "./WorkbenchRightRail";
export { WorkbenchMainContent } from "./WorkbenchMainContent";
export type { WorkbenchMainContentProps } from "./WorkbenchMainContent";

export interface WorkbenchQuickAction {
  key: string;
  label: string;
  icon: LucideIcon;
  onClick: () => void;
  disabled?: boolean;
}
