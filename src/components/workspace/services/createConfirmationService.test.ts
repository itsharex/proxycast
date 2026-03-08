import { describe, expect, it } from "vitest";
import {
  consumePendingCreateConfirmationMap,
  resolveContinuationTargetContent,
  resolveCreateConfirmationDecision,
  upsertPendingCreateConfirmationMap,
} from "./createConfirmationService";

describe("createConfirmationService", () => {
  it("upsertPendingCreateConfirmationMap 应写入确认状态", () => {
    const next = upsertPendingCreateConfirmationMap({}, "project-1", {
      source: "workspace_prompt",
      defaultCreationMode: "guided",
      initialUserPrompt: "  请生成标题  ",
      preferredContentId: "content-1",
    });

    expect(next["project-1"]?.source).toBe("workspace_prompt");
    expect(next["project-1"]?.creationMode).toBe("guided");
    expect(next["project-1"]?.initialUserPrompt).toBe("请生成标题");
    expect(next["project-1"]?.preferredContentId).toBe("content-1");
  });

  it("consumePendingCreateConfirmationMap 应移除指定项目确认状态", () => {
    const previous = upsertPendingCreateConfirmationMap({}, "project-1", {
      source: "workspace_prompt",
      defaultCreationMode: "guided",
    });
    const next = consumePendingCreateConfirmationMap(previous, "project-1");
    expect(next["project-1"]).toBeUndefined();
  });

  it("resolveContinuationTargetContent 应优先命中指定文稿", () => {
    const target = resolveContinuationTargetContent(
      [
        { id: "content-older", updated_at: 1 },
        { id: "content-newer", updated_at: 2 },
      ],
      "content-older",
    );
    expect(target?.id).toBe("content-older");
  });

  it("resolveCreateConfirmationDecision 继续历史时应返回 continue_history", () => {
    const pending = upsertPendingCreateConfirmationMap({}, "project-1", {
      source: "open_project_for_writing",
      defaultCreationMode: "guided",
      initialUserPrompt: "请先补充一个大纲",
      preferredContentId: "content-2",
    })["project-1"];

    const result = resolveCreateConfirmationDecision({
      pending,
      formData: {
        create_confirmation_option: ["continue_history"],
        create_confirmation_note: "",
      },
      defaultContentTitle: "新文稿",
    });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      return;
    }
    expect(result.decision.type).toBe("continue_history");
    expect(result.decision.initialUserPrompt).toContain("大纲");
    expect((result.decision as Extract<typeof result.decision, { type: "continue_history" }>).preferredContentId).toBe("content-2");
  });

  it("resolveCreateConfirmationDecision 新建时应返回 create_new", () => {
    const pending = upsertPendingCreateConfirmationMap({}, "project-1", {
      source: "workspace_prompt",
      defaultCreationMode: "guided",
      creationMode: "hybrid",
      initialUserPrompt: "帮我生成短视频脚本",
    })["project-1"];

    const result = resolveCreateConfirmationDecision({
      pending,
      formData: {
        create_confirmation_option: ["new_post"],
        create_confirmation_note: "",
      },
      defaultContentTitle: "新文稿",
    });

    expect(result.ok).toBe(true);
    if (!result.ok) {
      return;
    }
    expect(result.decision.type).toBe("create_new");
    if (result.decision.type !== "create_new") {
      return;
    }
    expect(result.decision.creationMode).toBe("hybrid");
    expect(result.decision.initialUserPrompt).toContain("短视频脚本");
    expect(result.decision.metadata.createConfirmation).toBeTruthy();
  });
});
