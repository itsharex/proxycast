/**
 * @file OAuth Provider 插件容器组件
 * @description 专门用于 OAuth Provider 插件的容器，整合 SDK 和 UI 系统
 * @module components/plugins/OAuthPluginContainer
 */

import React, { useState, useCallback } from "react";
import {
  Loader2,
  AlertCircle,
  RefreshCw,
  Settings2,
  Key,
  Plus,
  FileText,
  FolderOpen,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Modal } from "@/components/Modal";
import { usePluginSDK } from "@/lib/plugin-sdk";
import { PluginUIContainer } from "@/lib/plugin-ui";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { PluginId } from "@/lib/plugin-sdk/types";
import type { CredentialInfo } from "@/lib/plugin-sdk/types";

interface OAuthPluginContainerProps {
  /** 插件 ID */
  pluginId: PluginId;
  /** 插件显示名称 */
  displayName: string;
  /** 插件描述 */
  description?: string;
  /** 插件版本 */
  version?: string;
  /** 是否启用 */
  enabled?: boolean;
  /** 自定义类名 */
  className?: string;
  /** 启用/禁用回调 */
  onToggleEnabled?: (enabled: boolean) => void;
}

/**
 * 凭证卡片组件
 */
const CredentialCard: React.FC<{
  credential: CredentialInfo;
  onRefresh: () => void;
  onDelete: () => void;
}> = ({ credential, onRefresh, onDelete }) => {
  const statusColors: Record<string, string> = {
    active: "bg-green-500",
    inactive: "bg-gray-500",
    expired: "bg-yellow-500",
    error: "bg-red-500",
  };

  return (
    <Card className="relative">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="text-base">
            {credential.displayName || credential.id}
          </CardTitle>
          <Badge variant="outline" className="text-xs">
            <span
              className={`w-2 h-2 rounded-full mr-1 ${statusColors[credential.status] || "bg-gray-500"}`}
            />
            {credential.status}
          </Badge>
        </div>
        <CardDescription className="text-xs">
          认证类型: {credential.authType}
        </CardDescription>
      </CardHeader>
      <CardContent className="pt-2">
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>
            {credential.lastUsedAt
              ? `最后使用: ${new Date(credential.lastUsedAt).toLocaleDateString()}`
              : "未使用"}
          </span>
        </div>
        <div className="flex gap-2 mt-3">
          <Button variant="outline" size="sm" onClick={onRefresh}>
            <RefreshCw className="h-3 w-3 mr-1" />
            刷新
          </Button>
          <Button variant="destructive" size="sm" onClick={onDelete}>
            删除
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

// ============================================================================
// 添加凭证模态框
// ============================================================================

type AddCredentialMode = "json" | "file";

interface AddCredentialDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  pluginId: string;
  pluginName: string;
  onAdd: (authType: string, config: Record<string, unknown>) => Promise<void>;
}

const AddCredentialDialog: React.FC<AddCredentialDialogProps> = ({
  open,
  onOpenChange,
  pluginName,
  onAdd,
}) => {
  const [mode, setMode] = useState<AddCredentialMode>("json");
  const [name, setName] = useState("");
  const [jsonContent, setJsonContent] = useState("");
  const [filePath, setFilePath] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 重置表单
  const resetForm = useCallback(() => {
    setName("");
    setJsonContent("");
    setFilePath("");
    setError(null);
    setLoading(false);
  }, []);

  // 关闭时重置
  const handleClose = useCallback(() => {
    resetForm();
    onOpenChange(false);
  }, [resetForm, onOpenChange]);

  // 选择文件
  const handleSelectFile = async () => {
    try {
      const selected = await openFileDialog({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (selected) {
        setFilePath(selected as string);
      }
    } catch (e) {
      console.error("Failed to open file dialog:", e);
    }
  };

  // 提交 JSON 模式
  const handleJsonSubmit = async () => {
    if (!jsonContent.trim()) {
      setError("请粘贴凭证 JSON 内容");
      return;
    }

    // 验证 JSON 格式
    let parsedConfig: Record<string, unknown>;
    try {
      parsedConfig = JSON.parse(jsonContent);
    } catch {
      setError("JSON 格式无效，请检查内容");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // 添加显示名称到配置
      if (name.trim()) {
        parsedConfig.displayName = name.trim();
      }
      await onAdd("oauth", parsedConfig);
      handleClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // 提交文件模式
  const handleFileSubmit = async () => {
    if (!filePath) {
      setError("请选择凭证文件");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // 读取文件内容并解析
      const config: Record<string, unknown> = {
        filePath,
        displayName: name.trim() || undefined,
      };
      await onAdd("oauth", config);
      handleClose();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Modal isOpen={open} onClose={handleClose} maxWidth="max-w-md">
      {/* Header */}
      <div className="flex items-center justify-between border-b px-6 py-4">
        <h3 className="text-lg font-semibold">添加 {pluginName} 凭证</h3>
      </div>

      {/* Content */}
      <div className="space-y-4 px-6 py-4">
        {/* 模式选择器 */}
        <div className="grid grid-cols-2 gap-1 p-1 bg-muted/50 rounded-xl border">
          <button
            type="button"
            onClick={() => {
              setMode("json");
              setError(null);
            }}
            disabled={loading}
            className={`py-2 px-3 text-sm rounded-lg transition-all duration-200 font-medium ${
              mode === "json"
                ? "bg-background text-foreground shadow-sm ring-1 ring-black/5"
                : "text-muted-foreground hover:text-foreground hover:bg-background/50"
            }`}
          >
            <FileText className="inline h-4 w-4 mr-1" />
            粘贴 JSON
          </button>
          <button
            type="button"
            onClick={() => {
              setMode("file");
              setError(null);
            }}
            disabled={loading}
            className={`py-2 px-3 text-sm rounded-lg transition-all duration-200 font-medium ${
              mode === "file"
                ? "bg-background text-foreground shadow-sm ring-1 ring-black/5"
                : "text-muted-foreground hover:text-foreground hover:bg-background/50"
            }`}
          >
            <FolderOpen className="inline h-4 w-4 mr-1" />
            导入文件
          </button>
        </div>

        {/* 名称字段 */}
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

        {/* JSON 模式 */}
        {mode === "json" && (
          <div>
            <label className="mb-1 block text-sm font-medium">
              凭证 JSON <span className="text-red-500">*</span>
            </label>
            <textarea
              value={jsonContent}
              onChange={(e) => setJsonContent(e.target.value)}
              placeholder={`粘贴凭证 JSON 内容，例如：
{
  "accessToken": "...",
  "refreshToken": "...",
  ...
}`}
              disabled={loading}
              className="w-full h-48 rounded-lg border bg-background px-3 py-2 text-sm font-mono resize-none"
            />
          </div>
        )}

        {/* 文件模式 */}
        {mode === "file" && (
          <div>
            <label className="mb-1 block text-sm font-medium">
              凭证文件路径 <span className="text-red-500">*</span>
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
                placeholder="选择凭证文件..."
                disabled={loading}
                className="flex-1 rounded-lg border bg-background px-3 py-2 text-sm"
              />
              <button
                type="button"
                onClick={handleSelectFile}
                disabled={loading}
                className="flex items-center gap-1 rounded-lg border px-3 py-2 text-sm hover:bg-muted"
              >
                <FolderOpen className="h-4 w-4" />
                浏览
              </button>
            </div>
          </div>
        )}

        {/* 错误提示 */}
        {error && (
          <div className="rounded-lg border border-red-500 bg-red-50 p-3 text-sm text-red-700 dark:bg-red-950/30">
            {error}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="flex justify-end gap-2 border-t px-6 py-4">
        <button
          onClick={handleClose}
          disabled={loading}
          className="rounded-lg border px-4 py-2 text-sm hover:bg-muted"
        >
          取消
        </button>
        <button
          onClick={mode === "json" ? handleJsonSubmit : handleFileSubmit}
          disabled={loading}
          className="rounded-lg bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
        >
          {loading ? "添加中..." : "添加凭证"}
        </button>
      </div>
    </Modal>
  );
};

/**
 * OAuth Provider 插件容器
 *
 * 提供：
 * - 凭证列表管理
 * - 插件设置
 * - 插件 UI 渲染
 */
export const OAuthPluginContainer: React.FC<OAuthPluginContainerProps> = ({
  pluginId,
  displayName,
  description,
  version,
  enabled = true,
  className,
  onToggleEnabled,
}) => {
  const { sdk, credentials, loading, error, refresh } = usePluginSDK(pluginId);
  const [activeTab, setActiveTab] = useState("credentials");
  const [addCredentialDialogOpen, setAddCredentialDialogOpen] = useState(false);

  // 添加凭证
  const handleAddCredential = useCallback(
    async (authType: string, config: Record<string, unknown>) => {
      try {
        await sdk.credential.create(authType, config);
        sdk.notification.success("凭证添加成功");
        refresh();
      } catch (e) {
        const msg = e instanceof Error ? e.message : "未知错误";
        sdk.notification.error(`添加失败: ${msg}`);
        throw e; // 重新抛出以便模态框显示错误
      }
    },
    [sdk, refresh],
  );

  // 刷新凭证
  const handleRefreshCredential = useCallback(
    async (credentialId: string) => {
      try {
        await sdk.credential.refresh(credentialId);
        sdk.notification.success("凭证刷新成功");
        refresh();
      } catch (e) {
        const msg = e instanceof Error ? e.message : "未知错误";
        sdk.notification.error(`刷新失败: ${msg}`);
      }
    },
    [sdk, refresh],
  );

  // 删除凭证
  const handleDeleteCredential = useCallback(
    async (credentialId: string) => {
      try {
        await sdk.credential.delete(credentialId);
        sdk.notification.success("凭证已删除");
        refresh();
      } catch (e) {
        const msg = e instanceof Error ? e.message : "未知错误";
        sdk.notification.error(`删除失败: ${msg}`);
      }
    },
    [sdk, refresh],
  );

  // 加载状态
  if (loading) {
    return (
      <div className={`flex items-center justify-center p-8 ${className}`}>
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        <span className="ml-2 text-muted-foreground">加载插件...</span>
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div
        className={`flex flex-col items-center justify-center p-8 ${className}`}
      >
        <AlertCircle className="h-8 w-8 text-red-500 mb-2" />
        <p className="text-red-600 mb-4">{error}</p>
        <Button variant="outline" size="sm" onClick={refresh}>
          <RefreshCw className="h-4 w-4 mr-2" />
          重试
        </Button>
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* 插件头部信息 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                {displayName}
                {version && (
                  <Badge variant="secondary" className="text-xs">
                    v{version}
                  </Badge>
                )}
              </CardTitle>
              {description && (
                <CardDescription className="mt-1">
                  {description}
                </CardDescription>
              )}
            </div>
            <div className="flex items-center gap-2">
              <Badge variant={enabled ? "default" : "secondary"}>
                {enabled ? "已启用" : "已禁用"}
              </Badge>
              {onToggleEnabled && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => onToggleEnabled(!enabled)}
                >
                  {enabled ? "禁用" : "启用"}
                </Button>
              )}
            </div>
          </div>
        </CardHeader>
      </Card>

      {/* 标签页 */}
      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="credentials" className="flex items-center gap-1">
            <Key className="h-4 w-4" />
            凭证 ({credentials.length})
          </TabsTrigger>
          <TabsTrigger value="ui" className="flex items-center gap-1">
            插件 UI
          </TabsTrigger>
          <TabsTrigger value="settings" className="flex items-center gap-1">
            <Settings2 className="h-4 w-4" />
            设置
          </TabsTrigger>
        </TabsList>

        {/* 凭证列表 */}
        <TabsContent value="credentials" className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-sm font-medium">已配置的凭证</h3>
            <Button size="sm" onClick={() => setAddCredentialDialogOpen(true)}>
              <Plus className="h-4 w-4 mr-1" />
              添加凭证
            </Button>
          </div>

          {credentials.length === 0 ? (
            <div className="flex flex-col items-center justify-center p-8 text-muted-foreground border rounded-lg border-dashed">
              <Key className="h-8 w-8 mb-2" />
              <p>暂无凭证</p>
              <p className="text-xs mt-1">点击"添加凭证"开始配置</p>
              <Button
                variant="outline"
                size="sm"
                className="mt-4"
                onClick={() => setAddCredentialDialogOpen(true)}
              >
                <Plus className="h-4 w-4 mr-1" />
                添加第一个凭证
              </Button>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2">
              {credentials.map((cred) => (
                <CredentialCard
                  key={cred.id}
                  credential={cred}
                  onRefresh={() => handleRefreshCredential(cred.id)}
                  onDelete={() => handleDeleteCredential(cred.id)}
                />
              ))}
            </div>
          )}
        </TabsContent>

        {/* 插件 UI */}
        <TabsContent value="ui">
          <PluginUIContainer
            pluginId={pluginId}
            emptyMessage="该插件没有提供自定义 UI"
          />
        </TabsContent>

        {/* 设置 */}
        <TabsContent value="settings">
          <Card>
            <CardHeader>
              <CardTitle className="text-base">插件设置</CardTitle>
              <CardDescription>配置插件的行为和参数</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-muted-foreground text-sm">
                此插件暂无可配置的设置项
              </p>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* 添加凭证模态框 */}
      <AddCredentialDialog
        open={addCredentialDialogOpen}
        onOpenChange={setAddCredentialDialogOpen}
        pluginId={pluginId}
        pluginName={displayName}
        onAdd={handleAddCredential}
      />
    </div>
  );
};

export default OAuthPluginContainer;
