/**
 * Antigravity 凭证添加表单（自包含版本）
 *
 * 这是 AntigravityForm 的包装组件，内部管理所有状态，
 * 适合在插件中独立使用。
 *
 * @module components/provider-pool/credential-forms/AntigravityFormStandalone
 */

import { useState, useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { AntigravityForm } from "./AntigravityForm";
import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

/** Antigravity 凭证文件默认路径 */
const ANTIGRAVITY_DEFAULT_CREDS_PATH = "~/.antigravity/oauth_creds.json";

interface AntigravityFormStandaloneProps {
  /** 添加成功回调 */
  onSuccess: () => void;
  /** 取消回调 */
  onCancel?: () => void;
  /** 初始名称 */
  initialName?: string;
}

/**
 * 自包含的 Antigravity 凭证添加表单
 *
 * 内部管理所有状态，只需要提供 onSuccess 和 onCancel 回调
 */
export function AntigravityFormStandalone({
  onSuccess,
  onCancel,
  initialName = "",
}: AntigravityFormStandaloneProps) {
  const [name, setName] = useState(initialName);
  const [credsFilePath, setCredsFilePath] = useState(
    ANTIGRAVITY_DEFAULT_CREDS_PATH,
  );
  const [projectId, setProjectId] = useState("");
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

  // 使用 AntigravityForm hook 获取渲染函数和提交方法
  const antigravityForm = AntigravityForm({
    name,
    credsFilePath,
    setCredsFilePath,
    projectId,
    setProjectId,
    onSelectFile: handleSelectFile,
    loading,
    setLoading,
    setError,
    onSuccess: handleSuccess,
  });

  // 处理提交
  const handleSubmit = useCallback(() => {
    if (antigravityForm.mode === "file") {
      antigravityForm.handleFileSubmit();
    } else if (antigravityForm.mode === "login") {
      antigravityForm.handleGetAuthUrl();
    }
  }, [antigravityForm]);

  // 是否显示提交按钮
  const showSubmitButton = antigravityForm.mode === "file";
  // login 模式显示获取授权 URL 按钮
  const showLoginButton =
    antigravityForm.mode === "login" && !antigravityForm.waitingForCallback;

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

      {/* Antigravity 表单内容 */}
      {antigravityForm.render()}

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
        {showLoginButton && (
          <Button type="button" onClick={handleSubmit} disabled={loading}>
            {loading ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                获取中...
              </>
            ) : (
              "获取授权 URL"
            )}
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

export default AntigravityFormStandalone;
