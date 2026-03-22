import { describe, expect, it } from "vitest";
import { parseImageWorkbenchCommand } from "./imageWorkbenchCommand";

describe("parseImageWorkbenchCommand", () => {
  it("应解析生成命令的数量与比例", () => {
    const result = parseImageWorkbenchCommand(
      "@配图 生成 公众号头图，科技感，16:9，出 4 张",
    );

    expect(result).toMatchObject({
      trigger: "@配图",
      mode: "generate",
      count: 4,
      aspectRatio: "16:9",
      size: "1792x1024",
      prompt: "公众号头图，科技感",
    });
  });

  it("应解析编辑命令的目标图引用", () => {
    const result = parseImageWorkbenchCommand(
      "@image 编辑 #img-2 去掉文字，保留主体",
    );

    expect(result).toMatchObject({
      trigger: "@image",
      mode: "edit",
      targetRef: "img-2",
      prompt: "去掉文字，保留主体",
    });
  });

  it("未显式声明动作但带目标图时应默认为变体", () => {
    const result = parseImageWorkbenchCommand(
      "/image #img-7 更偏插画风，4:5",
    );

    expect(result).toMatchObject({
      trigger: "/image",
      mode: "variation",
      targetRef: "img-7",
      aspectRatio: "4:5",
      size: "864x1152",
      prompt: "更偏插画风",
    });
  });

  it("非图片命令应返回空", () => {
    expect(parseImageWorkbenchCommand("帮我总结一下这段代码")).toBeNull();
  });
});
