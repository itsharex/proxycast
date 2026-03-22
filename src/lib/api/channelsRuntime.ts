import { safeInvoke } from "@/lib/dev-bridge";
import type {
  CloudflaredInstallResult,
  CloudflaredInstallStatus,
  DiscordProbeResult,
  FeishuProbeResult,
  GatewayChannelStatusResponse,
  GatewayTunnelCreateResponse,
  GatewayTunnelProbeResult,
  GatewayTunnelStatus,
  GatewayTunnelSyncWebhookResponse,
  TelegramProbeResult,
  WechatConfiguredAccount,
  WechatLoginStartResult,
  WechatLoginWaitResult,
  WechatProbeResult,
} from "./channelsRuntimeTypes";

export type {
  ChannelsConfig,
  CloudflaredInstallResult,
  CloudflaredInstallStatus,
  DiscordAccountConfig,
  DiscordActionsConfig,
  DiscordAgentComponentsConfig,
  DiscordBotConfig,
  DiscordChannelConfig,
  DiscordExecApprovalsConfig,
  DiscordGatewayAccountStatus,
  DiscordGatewayStatus,
  DiscordGuildConfig,
  DiscordIntentsConfig,
  DiscordProbeResult,
  DiscordThreadBindingsConfig,
  DiscordUiComponentsConfig,
  DiscordUiConfig,
  DiscordVoiceAutoJoinConfig,
  DiscordVoiceConfig,
  DiscordAutoPresenceConfig,
  FeishuBotConfig,
  FeishuAccountConfig,
  FeishuGatewayAccountStatus,
  FeishuGatewayStatus,
  FeishuGroupConfig,
  FeishuProbeResult,
  CloudflareTunnelConfig,
  GatewayChannelStatusResponse,
  GatewayConfig,
  GatewayTunnelCreateResponse,
  GatewayTunnelConfig,
  GatewayTunnelProbeResult,
  GatewayTunnelStatus,
  GatewayTunnelSyncWebhookResponse,
  TelegramBotConfig,
  TelegramGatewayAccountStatus,
  TelegramGatewayStatus,
  TelegramProbeResult,
  WechatAccountConfig,
  WechatBotConfig,
  WechatConfiguredAccount,
  WechatGatewayAccountStatus,
  WechatGatewayStatus,
  WechatGroupConfig,
  WechatLoginStartResult,
  WechatLoginWaitResult,
  WechatProbeResult,
} from "./channelsRuntimeTypes";

export async function gatewayChannelStart(params?: {
  channel?: "telegram" | "feishu" | "discord" | "wechat";
  accountId?: string;
  pollTimeoutSecs?: number;
}): Promise<GatewayChannelStatusResponse> {
  return safeInvoke("gateway_channel_start", {
    request: {
      channel: params?.channel ?? "telegram",
      account_id: params?.accountId?.trim() || undefined,
      poll_timeout_secs: params?.pollTimeoutSecs,
    },
  });
}

export async function gatewayChannelStop(params?: {
  channel?: "telegram" | "feishu" | "discord" | "wechat";
  accountId?: string;
}): Promise<GatewayChannelStatusResponse> {
  return safeInvoke("gateway_channel_stop", {
    request: {
      channel: params?.channel ?? "telegram",
      account_id: params?.accountId?.trim() || undefined,
    },
  });
}

export async function gatewayChannelStatus(params?: {
  channel?: "telegram" | "feishu" | "discord" | "wechat";
}): Promise<GatewayChannelStatusResponse> {
  return safeInvoke("gateway_channel_status", {
    request: {
      channel: params?.channel ?? "telegram",
    },
  });
}

export async function telegramChannelProbe(params?: {
  accountId?: string;
}): Promise<TelegramProbeResult> {
  return safeInvoke("telegram_channel_probe", {
    request: {
      account_id: params?.accountId?.trim() || undefined,
    },
  });
}

export async function feishuChannelProbe(params?: {
  accountId?: string;
}): Promise<FeishuProbeResult> {
  return safeInvoke("feishu_channel_probe", {
    request: {
      account_id: params?.accountId?.trim() || undefined,
    },
  });
}

export async function discordChannelProbe(params?: {
  accountId?: string;
}): Promise<DiscordProbeResult> {
  return safeInvoke("discord_channel_probe", {
    request: {
      account_id: params?.accountId?.trim() || undefined,
    },
  });
}

export async function wechatChannelProbe(params?: {
  accountId?: string;
}): Promise<WechatProbeResult> {
  return safeInvoke("wechat_channel_probe", {
    request: {
      account_id: params?.accountId?.trim() || undefined,
    },
  });
}

export async function wechatChannelLoginStart(params?: {
  baseUrl?: string;
  botType?: string;
  sessionKey?: string;
}): Promise<WechatLoginStartResult> {
  return safeInvoke("wechat_channel_login_start", {
    request: {
      base_url: params?.baseUrl?.trim() || undefined,
      bot_type: params?.botType?.trim() || undefined,
      session_key: params?.sessionKey?.trim() || undefined,
    },
  });
}

export async function wechatChannelLoginWait(params: {
  sessionKey: string;
  baseUrl?: string;
  botType?: string;
  timeoutMs?: number;
  accountName?: string;
}): Promise<WechatLoginWaitResult> {
  return safeInvoke("wechat_channel_login_wait", {
    request: {
      session_key: params.sessionKey,
      base_url: params.baseUrl?.trim() || undefined,
      bot_type: params.botType?.trim() || undefined,
      timeout_ms: params.timeoutMs,
      account_name: params.accountName?.trim() || undefined,
    },
  });
}

export async function wechatChannelListAccounts(): Promise<
  WechatConfiguredAccount[]
> {
  return safeInvoke("wechat_channel_list_accounts");
}

export async function wechatChannelRemoveAccount(params: {
  accountId: string;
  purgeData?: boolean;
}): Promise<void> {
  return safeInvoke("wechat_channel_remove_account", {
    request: {
      account_id: params.accountId,
      purge_data: params.purgeData ?? false,
    },
  });
}

export async function wechatChannelSetRuntimeModel(params: {
  providerId: string;
  modelId: string;
}): Promise<string> {
  return safeInvoke("wechat_channel_set_runtime_model", {
    request: {
      provider_id: params.providerId.trim(),
      model_id: params.modelId.trim(),
    },
  });
}

export async function gatewayTunnelProbe(): Promise<GatewayTunnelProbeResult> {
  return safeInvoke("gateway_tunnel_probe");
}

export async function gatewayTunnelDetectCloudflared(): Promise<CloudflaredInstallStatus> {
  return safeInvoke("gateway_tunnel_detect_cloudflared");
}

export async function gatewayTunnelInstallCloudflared(params?: {
  confirm?: boolean;
}): Promise<CloudflaredInstallResult> {
  return safeInvoke("gateway_tunnel_install_cloudflared", {
    request: {
      confirm: params?.confirm ?? false,
    },
  });
}

export async function gatewayTunnelCreate(params?: {
  tunnelName?: string;
  dnsName?: string;
  persist?: boolean;
}): Promise<GatewayTunnelCreateResponse> {
  return safeInvoke("gateway_tunnel_create", {
    request: {
      tunnel_name: params?.tunnelName?.trim() || undefined,
      dns_name: params?.dnsName?.trim() || undefined,
      persist: params?.persist ?? true,
    },
  });
}

export async function gatewayTunnelStart(): Promise<GatewayTunnelStatus> {
  return safeInvoke("gateway_tunnel_start");
}

export async function gatewayTunnelStop(): Promise<GatewayTunnelStatus> {
  return safeInvoke("gateway_tunnel_stop");
}

export async function gatewayTunnelRestart(): Promise<GatewayTunnelStatus> {
  return safeInvoke("gateway_tunnel_restart");
}

export async function gatewayTunnelStatus(): Promise<GatewayTunnelStatus> {
  return safeInvoke("gateway_tunnel_status");
}

export async function gatewayTunnelSyncWebhookUrl(params: {
  channel: "feishu";
  accountId?: string;
  webhookPath?: string;
  persist?: boolean;
}): Promise<GatewayTunnelSyncWebhookResponse> {
  return safeInvoke("gateway_tunnel_sync_webhook_url", {
    request: {
      channel: params.channel,
      account_id: params.accountId?.trim() || undefined,
      webhook_path: params.webhookPath?.trim() || undefined,
      persist: params.persist ?? true,
    },
  });
}
