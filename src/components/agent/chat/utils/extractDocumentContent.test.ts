import { describe, expect, it } from "vitest";

/**
 * 测试 extractDocumentContent 函数的逻辑
 * 注意：这是一个独立的测试文件，用于验证内容提取逻辑
 */

// 模拟 extractDocumentContent 函数的逻辑
function extractDocumentContent(
  content: string,
  isThemeWorkbench: boolean,
): string | null {
  // 1. 检查 <document> 标签
  const documentMatch = content.match(/<document>([\s\S]*?)<\/document>/);
  if (documentMatch) {
    return documentMatch[1].trim();
  }

  // 2. 检查 markdown 代码块
  const markdownMatch = content.match(/```(?:markdown|md)\n([\s\S]*?)```/);
  if (markdownMatch) {
    return markdownMatch[1].trim();
  }

  // 3. 主题工作台：不使用启发式规则，避免误判普通回复
  if (isThemeWorkbench) {
    return null;
  }

  // 4. 非主题工作台：如果整个内容以 # 开头且长度超过 200 字符，认为是文档
  if (content.trim().startsWith("#") && content.length > 200) {
    return content.trim();
  }

  return null;
}

describe("extractDocumentContent", () => {
  describe("明确标记的内容", () => {
    it("应提取 <document> 标签内的内容", () => {
      const content = `
这是一些说明文字
<document>
# 文档标题
这是文档内容
</document>
更多说明
      `;
      expect(extractDocumentContent(content, true)).toBe(
        "# 文档标题\n这是文档内容",
      );
      expect(extractDocumentContent(content, false)).toBe(
        "# 文档标题\n这是文档内容",
      );
    });

    it("应提取 markdown 代码块内的内容", () => {
      const content = `
这是一些说明文字
\`\`\`markdown
# 文档标题
这是文档内容
\`\`\`
更多说明
      `;
      expect(extractDocumentContent(content, true)).toBe(
        "# 文档标题\n这是文档内容",
      );
      expect(extractDocumentContent(content, false)).toBe(
        "# 文档标题\n这是文档内容",
      );
    });
  });

  describe("主题工作台模式", () => {
    it("不应提取普通对话（即使以 # 开头且较长）", () => {
      const longContent = `# 你说得对，这次是我搞错了
我理解你的意思了。让我重新分析一下这个问题。
${"这是一段很长的文本。".repeat(20)}`;

      expect(extractDocumentContent(longContent, true)).toBeNull();
    });

    it("不应提取没有明确标记的内容", () => {
      const content = "这是一段普通的对话回复，没有任何标记。";
      expect(extractDocumentContent(content, true)).toBeNull();
    });
  });

  describe("非主题工作台模式", () => {
    it("应提取以 # 开头且长度超过 200 字符的内容", () => {
      const longContent = `# 文档标题
${"这是一段很长的文本。".repeat(20)}`;

      expect(extractDocumentContent(longContent, false)).toBe(
        longContent.trim(),
      );
    });

    it("不应提取以 # 开头但长度不足 200 字符的内容", () => {
      const shortContent = "# 短标题\n这是一段短文本。";
      expect(extractDocumentContent(shortContent, false)).toBeNull();
    });

    it("不应提取不以 # 开头的内容", () => {
      const content = `${"这是一段很长的文本。".repeat(30)}`;
      expect(extractDocumentContent(content, false)).toBeNull();
    });
  });

  describe("边界情况", () => {
    it("应处理空字符串", () => {
      expect(extractDocumentContent("", true)).toBeNull();
      expect(extractDocumentContent("", false)).toBeNull();
    });

    it("应处理只有空白字符的字符串", () => {
      expect(extractDocumentContent("   \n\n   ", true)).toBeNull();
      expect(extractDocumentContent("   \n\n   ", false)).toBeNull();
    });

    it("应优先提取 <document> 标签而不是启发式规则", () => {
      const content = `
# 这是一段很长的文本
${"内容".repeat(100)}
<document>
# 真正的文档
这才是要提取的内容
</document>
      `;
      expect(extractDocumentContent(content, false)).toBe(
        "# 真正的文档\n这才是要提取的内容",
      );
    });
  });
});
