import { useState, useEffect } from "react";
import { X, Eye, EyeOff, Settings, FolderOpen, Upload, CheckCircle } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  CredentialDisplay,
  UpdateCredentialRequest,
} from "@/lib/api/providerPool";

interface EditCredentialModalProps {
  credential: CredentialDisplay | null;
  isOpen: boolean;
  onClose: () => void;
  onEdit: (uuid: string, request: UpdateCredentialRequest) => Promise<void>;
}

export function EditCredentialModal({
  credential,
  isOpen,
  onClose,
  onEdit,
}: EditCredentialModalProps) {
  const [name, setName] = useState("");
  const [checkHealth, setCheckHealth] = useState(true);
  const [checkModelName, setCheckModelName] = useState("");
  const [notSupportedModelsText, setNotSupportedModelsText] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCredentialDetails, setShowCredentialDetails] = useState(false);

  // 重新上传文件相关状态
  const [newCredFilePath, setNewCredFilePath] = useState("");
  const [newProjectId, setNewProjectId] = useState("");

  // 初始化表单数据
  useEffect(() => {
    if (credential) {
      setName(credential.name || "");
      setCheckHealth(credential.check_health);
      setCheckModelName(credential.check_model_name || "");
      setNotSupportedModelsText(
        (credential.not_supported_models || []).join(", "),
      );
      setNewCredFilePath("");
      setNewProjectId("");
      setError(null);
    }
  }, [credential]);

  if (!isOpen || !credential) {
    return null;
  }

  const isOAuth = credential.credential_type.includes("oauth");

  const handleSelectNewFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (selected) {
        setNewCredFilePath(selected as string);
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  };

  const getMaskedCredentialInfo = () => {
    if (isOAuth) {
      // OAuth 凭证显示文件路径（部分遮罩）
      const path = credential.display_credential;
      const parts = path.split("/");
      if (parts.length > 1) {
        const fileName = parts[parts.length - 1];
        const dirPath = parts.slice(0, -1).join("/");
        return `${dirPath}/***${fileName.slice(-8)}`;
      }
      return `***${path.slice(-12)}`;
    } else {
      // API Key 显示遮罩
      return credential.display_credential;
    }
  };

  const handleSubmit = async () => {
    setLoading(true);
    setError(null);

    try {
      // 解析不支持的模型列表
      const parsedNotSupportedModels = notSupportedModelsText
        .split(",")
        .map((model) => model.trim())
        .filter((model) => model.length > 0);

      const updateRequest: UpdateCredentialRequest = {
        name: name.trim() || undefined,
        check_health: checkHealth,
        check_model_name: checkModelName.trim() || undefined,
        not_supported_models:
          parsedNotSupportedModels.length > 0
            ? parsedNotSupportedModels
            : undefined,
        new_creds_file_path: newCredFilePath.trim() || undefined,
        new_project_id: newProjectId.trim() || undefined,
      };

      await onEdit(credential.uuid, updateRequest);
      onClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
      <div className="w-full max-w-2xl h-[80vh] rounded-lg bg-background shadow-xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between border-b pb-4 px-6 pt-6">
          <h3 className="text-lg font-semibold flex items-center gap-2">
            <Settings className="h-5 w-5" />
            编辑凭证
          </h3>
          <button onClick={onClose} className="rounded-lg p-1 hover:bg-muted">
            <X className="h-5 w-5" />
          </button>
        </div>

        {/* Content - Scrollable */}
        <div className="flex-1 overflow-y-auto px-6 py-4">
          <div className="space-y-4">
          {/* 凭证信息（只读） */}
          <div className="rounded-lg bg-muted/50 p-3">
            <div className="flex items-center justify-between mb-2">
              <label className="text-sm font-medium">凭证信息</label>
              <button
                type="button"
                onClick={() => setShowCredentialDetails(!showCredentialDetails)}
                className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground"
              >
                {showCredentialDetails ? (
                  <>
                    <EyeOff className="h-3 w-3" />
                    隐藏
                  </>
                ) : (
                  <>
                    <Eye className="h-3 w-3" />
                    显示
                  </>
                )}
              </button>
            </div>
            <div className="space-y-2 text-sm">
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">类型:</span>
                <span className="font-mono">{credential.credential_type}</span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">UUID:</span>
                <span className="font-mono">
                  {credential.uuid.slice(0, 24)}...
                </span>
              </div>
              <div className="flex items-center justify-between">
                <span className="text-muted-foreground">
                  {isOAuth ? "文件路径:" : "API Key:"}
                </span>
                <span className="font-mono">
                  {showCredentialDetails
                    ? credential.display_credential
                    : getMaskedCredentialInfo()}
                </span>
              </div>
            </div>
            <p className="mt-2 text-xs text-muted-foreground">
              🔒 敏感信息（API Key、文件路径）无法修改，如需更改请删除后重新添加
            </p>
          </div>

          {/* 可编辑字段 */}
          <div>
            <label className="mb-1 block text-sm font-medium">名称</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="给这个凭证起个名字..."
              className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
            />
          </div>

          {/* 健康检查设置 */}
          <div>
            <label className="mb-2 flex items-center gap-2 text-sm font-medium">
              <input
                type="checkbox"
                checked={checkHealth}
                onChange={(e) => setCheckHealth(e.target.checked)}
                className="rounded"
              />
              启用自动健康检查
            </label>
            {checkHealth && (
              <div className="ml-6">
                <label className="mb-1 block text-xs font-medium text-muted-foreground">
                  检查模型（可选）
                </label>
                <input
                  type="text"
                  value={checkModelName}
                  onChange={(e) => setCheckModelName(e.target.value)}
                  placeholder="留空使用默认模型..."
                  className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
                />
              </div>
            )}
          </div>

          {/* 不支持的模型列表 */}
          <div>
            <label className="mb-1 block text-sm font-medium">
              不支持的模型
            </label>
            <textarea
              value={notSupportedModelsText}
              onChange={(e) => setNotSupportedModelsText(e.target.value)}
              placeholder="用逗号分隔多个模型，例如: model-1, model-2"
              className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
              rows={3}
            />
            <p className="mt-1 text-xs text-muted-foreground">
              这些模型将不会路由到此凭证
            </p>
          </div>

          {/* OAuth 文件重新上传 */}
          {isOAuth && (
            <div className="rounded-lg border border-amber-200 dark:border-amber-800 bg-amber-50/50 dark:bg-amber-950/30 p-4">
              <div className="flex items-center gap-2 mb-3">
                <div className="rounded-full bg-amber-100 dark:bg-amber-900/30 p-1.5">
                  <Upload className="h-3 w-3 text-amber-600 dark:text-amber-400" />
                </div>
                <span className="font-semibold text-sm">重新上传凭证文件</span>
              </div>
              <p className="text-xs text-muted-foreground mb-3">
                选择新的凭证文件来替换当前文件。新文件将被复制到应用存储目录。
              </p>
              <div className="space-y-3">
                <div>
                  <label className="mb-1 block text-xs font-medium text-muted-foreground">
                    新凭证文件
                  </label>
                  <div className="flex items-center gap-2">
                    <input
                      type="text"
                      value={newCredFilePath}
                      onChange={(e) => setNewCredFilePath(e.target.value)}
                      placeholder="选择新的凭证文件..."
                      className="flex-1 rounded-lg border bg-background px-3 py-2 text-sm"
                      readOnly
                    />
                    <button
                      type="button"
                      onClick={handleSelectNewFile}
                      className="flex items-center gap-1 rounded-lg bg-blue-100 px-3 py-2 text-xs font-medium text-blue-700 hover:bg-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:hover:bg-blue-800/40 transition-colors"
                    >
                      <FolderOpen className="h-3 w-3" />
                      选择文件
                    </button>
                  </div>
                </div>
                {credential.credential_type === "gemini_oauth" && (
                  <div>
                    <label className="mb-1 block text-xs font-medium text-muted-foreground">
                      项目ID（可选）
                    </label>
                    <input
                      type="text"
                      value={newProjectId}
                      onChange={(e) => setNewProjectId(e.target.value)}
                      placeholder="留空保持当前项目ID..."
                      className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
                    />
                  </div>
                )}
                {newCredFilePath && (
                  <div className="text-xs text-green-600 dark:text-green-400 flex items-center gap-1">
                    <CheckCircle className="h-3 w-3" />
                    文件已选择，保存后将替换当前凭证文件
                  </div>
                )}
              </div>
            </div>
          )}

          {/* 统计信息（只读） */}
          <div className="rounded-lg bg-muted/50 p-3">
            <label className="mb-2 block text-sm font-medium">使用统计</label>
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div className="flex justify-between">
                <span className="text-muted-foreground">使用次数:</span>
                <span className="font-mono">{credential.usage_count}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-muted-foreground">错误次数:</span>
                <span className="font-mono">{credential.error_count}</span>
              </div>
              <div className="col-span-2 flex justify-between">
                <span className="text-muted-foreground">最后使用:</span>
                <span className="text-xs">
                  {credential.last_used || "从未"}
                </span>
              </div>
            </div>
          </div>

          {/* Error */}
          {error && (
            <div className="rounded-lg border border-red-500 bg-red-50 p-3 text-sm text-red-700 dark:bg-red-950/30">
              {error}
            </div>
          )}
          </div>
        </div>

        {/* Footer */}
        <div className="border-t px-6 py-4 flex justify-end gap-2">
          <button
            onClick={onClose}
            className="rounded-lg border px-4 py-2 text-sm hover:bg-muted"
          >
            取消
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="rounded-lg bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {loading ? "保存中..." : "保存更改"}
          </button>
        </div>
      </div>
    </div>
  );
}
