/**
 * Kiro 凭证添加表单（自包含版本）
 *
 * 这是 KiroForm 的包装组件，内部管理所有状态，
 * 适合在插件中独立使用。
 *
 * @module components/provider-pool/credential-forms/KiroFormStandalone
 */

import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { KiroForm } from "./KiroForm";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

/** Kiro 凭证文件默认路径 */
const KIRO_DEFAULT_CREDS_PATH = "~/.aws/sso/cache/kiro-auth-token.json";

interface KiroFormStandaloneProps {
  /** 添加成功回调 */
  onSuccess: () => void;
  /** 取消回调 */
  onCancel?: () => void;
  /** 初始名称 */
  initialName?: string;
}

/**
 * 自包含的 Kiro 凭证添加表单
 *
 * 内部管理所有状态，只需要提供 onSuccess 和 onCancel 回调
 */
export function KiroFormStandalone({
  onSuccess,
  onCancel,
  initialName = "",
}: KiroFormStandaloneProps) {
  const [name, setName] = useState(initialName);
  // 设置默认凭证文件路径
  const [credsFilePath, setCredsFilePath] = useState(KIRO_DEFAULT_CREDS_PATH);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSelectFile = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (selected) {
        setCredsFilePath(selected as string);
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  }, []);

  const handleSuccess = useCallback(() => {
    onSuccess();
  }, [onSuccess]);

  // 使用 KiroForm hook 获取渲染函数和提交方法
  const kiroForm = KiroForm({
    name,
    credsFilePath,
    setCredsFilePath,
    onSelectFile: handleSelectFile,
    loading,
    setLoading,
    setError,
    onSuccess: handleSuccess,
  });

  // 处理提交
  const handleSubmit = useCallback(() => {
    if (kiroForm.mode === "json") {
      kiroForm.handleJsonSubmit();
    } else if (kiroForm.mode === "file") {
      kiroForm.handleFileSubmit();
    }
    // login 模式不需要手动提交，登录按钮会直接触发
  }, [kiroForm]);

  // 是否显示提交按钮（login 模式不需要）
  const showSubmitButton = kiroForm.mode !== "login";

  return (
    <div className="space-y-4">
      {/* 名称输入 */}
      <div>
        <label className="mb-1 block text-sm font-medium">名称 (可选)</label>
        <input
          type="text"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="给这个凭证起个名字..."
          disabled={loading}
          className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
        />
      </div>

      {/* Kiro 表单内容 */}
      {kiroForm.render()}

      {/* 错误提示 */}
      {error && (
        <div className="rounded-lg border border-red-300 bg-red-50 dark:bg-red-900/20 p-3 text-sm text-red-700 dark:text-red-300">
          {error}
        </div>
      )}

      {/* 按钮区域 */}
      <div className="flex justify-end gap-2 pt-2">
        {onCancel && (
          <Button
            type="button"
            variant="outline"
            onClick={onCancel}
            disabled={loading}
          >
            取消
          </Button>
        )}
        {showSubmitButton && (
          <Button type="button" onClick={handleSubmit} disabled={loading}>
            {loading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                添加中...
              </>
            ) : (
              "添加凭证"
            )}
          </Button>
        )}
      </div>
    </div>
  );
}

export default KiroFormStandalone;
