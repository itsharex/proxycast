import type { WorkbenchRightRailProps } from "./workbenchRightRailContracts";
import { WorkbenchRightRailActionSections } from "./workbenchRightRailActionSections";
import {
  WorkbenchRightRailCollapseBar,
  WorkbenchRightRailHeadingCard,
  WorkbenchRightRailStyleGuideCard,
  WorkbenchRightRailStyleGuideDialog,
} from "./workbenchRightRailExpandedChrome";
import { useWorkbenchRightRailCapabilityController } from "./useWorkbenchRightRailCapabilityController";
import type { WorkbenchRightRailCapabilitySection } from "./workbenchRightRailTypes";
import type { WorkspaceTheme } from "@/types/page";

export function WorkbenchRightRailExpandedPanel({
  onCollapse,
  projectId,
  contentId,
  onCreateContentFromPrompt,
  initialExpandedActionKey,
  onInitialExpandedActionConsumed,
  initialStyleGuideDialogOpen,
  onInitialStyleGuideDialogConsumed,
  initialStyleGuideSourceEntryId,
  onInitialStyleGuideSourceEntryConsumed,
  sections,
  heading,
  subheading,
  theme,
}: {
  onCollapse: () => void;
  projectId?: string | null;
  contentId?: string | null;
  onCreateContentFromPrompt?: WorkbenchRightRailProps["onCreateContentFromPrompt"];
  initialExpandedActionKey?: string | null;
  onInitialExpandedActionConsumed?: () => void;
  initialStyleGuideDialogOpen?: boolean;
  onInitialStyleGuideDialogConsumed?: () => void;
  initialStyleGuideSourceEntryId?: string | null;
  onInitialStyleGuideSourceEntryConsumed?: () => void;
  sections: WorkbenchRightRailCapabilitySection[];
  heading?: string | null;
  subheading?: string | null;
  theme?: WorkspaceTheme;
}) {
  const controller = useWorkbenchRightRailCapabilityController({
    projectId,
    contentId,
    initialExpandedActionKey,
    onInitialExpandedActionConsumed,
    initialStyleGuideDialogOpen,
    onInitialStyleGuideDialogConsumed,
    initialStyleGuideSourceEntryId,
    onInitialStyleGuideSourceEntryConsumed,
    onCreateContentFromPrompt,
  });

  return (
    <aside
      className="flex w-[320px] min-w-[320px] flex-col border-l bg-background/95"
      data-testid="workbench-right-rail-expanded"
    >
      <WorkbenchRightRailCollapseBar onCollapse={onCollapse} />

      <div className="flex flex-1 min-h-0 flex-col gap-4 overflow-y-auto px-3 py-3">
        <WorkbenchRightRailHeadingCard
          eyebrow={theme === "video" ? "视频助手" : undefined}
          heading={heading}
          subheading={subheading}
        />
        {theme === "video" ? null : (
          <WorkbenchRightRailStyleGuideCard
            projectId={projectId}
            onOpen={() => {
              controller.setStyleGuideSourceEntryId(null);
              controller.handleStyleGuideDialogOpenChange(true);
            }}
          />
        )}
        <WorkbenchRightRailActionSections
          sections={sections}
          controller={controller}
          theme={theme}
        />
      </div>

      <WorkbenchRightRailStyleGuideDialog
        open={controller.styleGuideDialogOpen}
        projectId={projectId}
        sourceEntryId={controller.styleGuideSourceEntryId}
        onOpenChange={controller.handleStyleGuideDialogOpenChange}
      />
    </aside>
  );
}
