import type { CreationMode } from "@/components/content-creator/types";
import type { WorkspaceTheme } from "@/types/page";

export interface WorkbenchRightRailProps {
  shouldRender: boolean;
  isCreateWorkspaceView: boolean;
  projectId?: string | null;
  contentId?: string | null;
  theme?: WorkspaceTheme;
  creationMode?: CreationMode;
  creationType?: string;
  initialStyleGuideDialogOpen?: boolean;
  onInitialStyleGuideDialogConsumed?: () => void;
  initialStyleGuideSourceEntryId?: string | null;
  onInitialStyleGuideSourceEntryConsumed?: () => void;
  onBackToCreateView: () => void;
  onCreateContentFromPrompt?: (prompt: string) => Promise<void> | void;
}
