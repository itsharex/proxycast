/**
 * @file 已配置 Provider 列表 Hook
 * @description 从 OAuth 凭证和 API Key Provider 中提取已配置的 Provider 列表
 * @module hooks/useConfiguredProviders
 */

import { useMemo } from "react";
import { useProviderPool } from "./useProviderPool";
import { useApiKeyProvider } from "./useApiKeyProvider";
import {
  getRegistryIdFromType,
  getProviderLabel,
} from "@/lib/constants/providerMappings";

// ============================================================================
// 类型定义
// ============================================================================

/**
 * 已配置的 Provider 信息
 */
export interface ConfiguredProvider {
  /** Provider 唯一标识 */
  key: string;
  /** 显示标签 */
  label: string;
  /** 模型注册表中的 provider_id */
  registryId: string;
  /** 回退的 registry_id（当 registryId 没有模型时使用） */
  fallbackRegistryId?: string;
  /** 原始 provider type，用于确定 API 协议 */
  type: string;
  /** 凭证类型（用于特殊处理） */
  credentialType?: string;
  /** Provider ID（用于 API Key Provider） */
  providerId?: string;
  /** 自定义模型列表（用于 API Key Provider） */
  customModels?: string[];
}

export interface UseConfiguredProvidersResult {
  /** 已配置的 Provider 列表 */
  providers: ConfiguredProvider[];
  /** 是否正在加载 */
  loading: boolean;
}

// ============================================================================
// Hook 实现
// ============================================================================

/**
 * 获取已配置的 Provider 列表
 *
 * 从 OAuth 凭证池和 API Key Provider 中提取已配置凭证的 Provider，
 * 合并去重后返回统一的 Provider 列表。
 *
 * @returns 已配置的 Provider 列表和加载状态
 *
 * @example
 * ```tsx
 * const { providers, loading } = useConfiguredProviders();
 *
 * if (loading) return <Spinner />;
 *
 * return (
 *   <select>
 *     {providers.map(p => (
 *       <option key={p.key} value={p.key}>{p.label}</option>
 *     ))}
 *   </select>
 * );
 * ```
 */
export function useConfiguredProviders(): UseConfiguredProvidersResult {
  // 获取凭证池数据
  const { overview: oauthCredentials, loading: oauthLoading } =
    useProviderPool();
  const { providers: apiKeyProviders, loading: apiKeyLoading } =
    useApiKeyProvider();

  // 计算已配置的 Provider 列表
  const providers = useMemo(() => {
    const safeOauthCredentials = Array.isArray(oauthCredentials)
      ? oauthCredentials
      : [];
    const safeApiKeyProviders = Array.isArray(apiKeyProviders)
      ? apiKeyProviders
      : [];
    const providerMap = new Map<string, ConfiguredProvider>();

    // 1. 从 OAuth 凭证提取 Provider
    safeOauthCredentials.forEach((overview) => {
      if (overview.credentials.length > 0) {
        const key = overview.provider_type;
        const firstCredential = overview.credentials[0];
        const credentialType = firstCredential.credential_type || key;

        if (!providerMap.has(key)) {
          providerMap.set(key, {
            key,
            label: getProviderLabel(key),
            registryId: getRegistryIdFromType(key),
            type: key,
            credentialType,
          });
        }
      }
    });

    // 2. 从 API Key Provider 提取
    // 使用 provider.id 作为 key，确保每个 Provider 单独显示
    // 特殊处理：如果与 OAuth 凭证冲突，使用带后缀的 key
    safeApiKeyProviders
      .filter((p) => p.api_key_count > 0 && p.enabled)
      .forEach((provider) => {
        let key = provider.id;
        let label = provider.name;

        // 如果 key 与 OAuth 凭证冲突，添加 "_api_key" 后缀
        // 例如：Gemini OAuth 的 key 是 "gemini"，Gemini API Key 的 key 变成 "gemini_api_key"
        if (providerMap.has(key)) {
          key = `${provider.id}_api_key`;
          label = `${provider.name} API Key`;
        }

        if (!providerMap.has(key)) {
          // 优先使用 provider.id 作为 registryId（适用于系统预设的 Provider，如 deepseek, moonshot）
          // 如果模型注册表中没有该 id 的模型，则回退到使用 type 映射（适用于自定义 Provider）
          providerMap.set(key, {
            key,
            label,
            registryId: provider.id,
            fallbackRegistryId: getRegistryIdFromType(provider.type),
            type: provider.type,
            credentialType: `${provider.type}_key`,
            providerId: provider.id,
            customModels: provider.custom_models,
          });
        }
      });

    return Array.from(providerMap.values());
  }, [oauthCredentials, apiKeyProviders]);

  return {
    providers,
    loading: oauthLoading || apiKeyLoading,
  };
}

export default useConfiguredProviders;
