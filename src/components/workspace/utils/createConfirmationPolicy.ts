import type { CreationMode } from "@/components/content-creator/types";
import type {
  A2UIFormData,
  A2UIResponse,
} from "@/components/content-creator/a2ui/types";

export type CreateConfirmationSource =
  | "project_created"
  | "open_project_for_writing"
  | "workspace_create_entry"
  | "workspace_prompt"
  | "quick_create";

export type CreateConfirmationOption =
  | "continue_history"
  | "new_post"
  | "new_version"
  | "other";

export interface PendingCreateConfirmation {
  projectId: string;
  source: CreateConfirmationSource;
  creationMode: CreationMode;
  initialUserPrompt?: string;
  preferredContentId?: string;
  fallbackContentTitle?: string;
  createdAt: number;
}

export interface CreateConfirmationIntent {
  option: CreateConfirmationOption;
  note: string;
}

interface ParseCreateConfirmationIntentSuccess {
  ok: true;
  intent: CreateConfirmationIntent;
}

interface ParseCreateConfirmationIntentFailure {
  ok: false;
  message: string;
}

export type ParseCreateConfirmationIntentResult =
  | ParseCreateConfirmationIntentSuccess
  | ParseCreateConfirmationIntentFailure;

export const CREATE_CONFIRMATION_FORM_FIELDS = {
  option: "create_confirmation_option",
  note: "create_confirmation_note",
} as const;

const SOURCE_HINTS: Record<CreateConfirmationSource, string> = {
  project_created: "已创建项目，请先确认本次是否需要新开文稿。",
  open_project_for_writing: "当前项目暂无文稿，请确认是继续历史还是新建。",
  workspace_create_entry: "请先确认创建意图，避免误建大量文稿。",
  workspace_prompt: "检测到快速提示词，请确认是否创建新文稿。",
  quick_create: "快捷创建前请再次确认，避免重复开稿。",
};

const OPTION_LABELS: Record<CreateConfirmationOption, string> = {
  continue_history: "仅继续历史文稿（不新建）",
  new_post: "新开帖子（创建新文稿）",
  new_version: "新建版本（创建新文稿）",
  other: "其他（需补充说明）",
};

function normalizeConfirmationOption(
  rawValue: unknown,
): CreateConfirmationOption | null {
  const firstValue = Array.isArray(rawValue) ? rawValue[0] : rawValue;
  if (typeof firstValue !== "string") {
    return null;
  }
  switch (firstValue) {
    case "continue_history":
    case "new_post":
    case "new_version":
    case "other":
      return firstValue;
    default:
      return null;
  }
}

export function parseCreateConfirmationIntent(
  formData: A2UIFormData,
): ParseCreateConfirmationIntentResult {
  const option = normalizeConfirmationOption(
    formData[CREATE_CONFIRMATION_FORM_FIELDS.option],
  );
  if (!option) {
    return {
      ok: false,
      message: "请选择本次处理方式",
    };
  }

  const noteRaw = formData[CREATE_CONFIRMATION_FORM_FIELDS.note];
  const note =
    typeof noteRaw === "string" ? noteRaw.trim() : String(noteRaw || "").trim();

  if (option === "other" && note.length < 2) {
    return {
      ok: false,
      message: "选择“其他”时请至少填写 2 个字说明",
    };
  }

  return {
    ok: true,
    intent: {
      option,
      note,
    },
  };
}

export function shouldCreateContentByIntent(
  intent: CreateConfirmationIntent,
): boolean {
  return intent.option !== "continue_history";
}

export function resolveConfirmedInitialPrompt(
  pending: PendingCreateConfirmation,
  intent: CreateConfirmationIntent,
): string {
  const preferredPrompt = pending.initialUserPrompt?.trim() || "";
  if (preferredPrompt) {
    return preferredPrompt;
  }
  if (intent.option === "other") {
    return intent.note;
  }
  return "";
}

export function resolveCreateContentTitle(
  pending: PendingCreateConfirmation,
  defaultTitle: string,
  intent: CreateConfirmationIntent,
): string {
  const fallbackTitle = pending.fallbackContentTitle?.trim() || "";
  if (fallbackTitle) {
    return fallbackTitle;
  }
  if (intent.option === "new_version") {
    return `新版本-${defaultTitle}`;
  }
  return defaultTitle;
}

export function buildCreateConfirmationMetadata(
  pending: PendingCreateConfirmation,
  intent: CreateConfirmationIntent,
): Record<string, unknown> {
  return {
    creationMode: pending.creationMode,
    createConfirmation: {
      source: pending.source,
      option: intent.option,
      optionLabel: OPTION_LABELS[intent.option],
      note: intent.note || null,
      confirmedAt: Date.now(),
    },
  };
}

export function buildCreateConfirmationA2UI(
  pending: PendingCreateConfirmation,
): A2UIResponse {
  const rootId = "create_confirmation_root";
  const titleId = "create_confirmation_title";
  const hintId = "create_confirmation_hint";
  const optionId = CREATE_CONFIRMATION_FORM_FIELDS.option;
  const noteId = CREATE_CONFIRMATION_FORM_FIELDS.note;

  return {
    id: `create-confirmation-${pending.projectId}`,
    root: rootId,
    data: {},
    components: [
      {
        id: titleId,
        component: "Text",
        text: "创建前确认",
        variant: "h3",
      },
      {
        id: hintId,
        component: "Text",
        text:
          SOURCE_HINTS[pending.source] ||
          "请先确认本次是否需要新建文稿，确认后才会生成。",
        variant: "caption",
      },
      {
        id: optionId,
        component: "ChoicePicker",
        label: "本次操作类型",
        value: ["new_post"],
        variant: "mutuallyExclusive",
        layout: "wrap",
        options: [
          {
            value: "continue_history",
            label: OPTION_LABELS.continue_history,
            description: "直接回到已有文稿，不新建",
          },
          {
            value: "new_post",
            label: OPTION_LABELS.new_post,
            description: "新开一个独立文稿",
          },
          {
            value: "new_version",
            label: OPTION_LABELS.new_version,
            description: "基于当前项目新增一个版本文稿",
          },
          {
            value: "other",
            label: OPTION_LABELS.other,
            description: "填写你的自定义意图",
          },
        ],
      },
      {
        id: noteId,
        component: "TextField",
        label: "补充说明（可选）",
        value: "",
        variant: "longText",
        placeholder: "如选择“其他”，请在这里说明你的创建意图",
        helperText: "不会自动生成，点击下方确认后才会执行。",
      },
      {
        id: rootId,
        component: "Column",
        children: [titleId, hintId, optionId, noteId],
        gap: 12,
        align: "stretch",
      },
    ],
    submitAction: {
      label: "确认生成",
      action: {
        name: "submit",
      },
    },
  };
}
