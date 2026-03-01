import { describe, expect, it } from "vitest";
import { sanitizeCrashValue } from "./crashReporting";

describe("crashReporting sanitizeCrashValue", () => {
  it("应脱敏常见密钥与 token", () => {
    const sanitized = sanitizeCrashValue({
      authorization: "Bearer sk-1234567890abcdef",
      apiKey: "api_key=sk-abcdef1234567890",
      token: "refresh_token=abcdefg1234567890",
      nested: {
        message: "access_token: abcdefghijklmnopqrst",
      },
    }) as Record<string, unknown>;

    expect(String(sanitized.authorization)).toContain("Bearer ***");
    expect(String(sanitized.apiKey)).toContain("api_key=***");
    expect(String(sanitized.token)).toContain("refresh_token=***");
    expect(String((sanitized.nested as Record<string, unknown>).message)).toContain(
      "access_token=***",
    );
  });

  it("应保留数字和布尔值结构", () => {
    const sanitized = sanitizeCrashValue({
      count: 3,
      ok: true,
      list: [1, false, "Bearer abcd1234"],
    }) as Record<string, unknown>;

    expect(sanitized.count).toBe(3);
    expect(sanitized.ok).toBe(true);
    expect(Array.isArray(sanitized.list)).toBe(true);
    expect(String((sanitized.list as unknown[])[2])).toContain("Bearer ***");
  });
});
