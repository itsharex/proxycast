//! 路由模型
//!
//! 用于多供应商路由功能的数据结构定义。

use serde::{Deserialize, Serialize};

/// 单个路由信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub selector: String,
    pub provider_type: String,
    pub credential_count: usize,
    pub endpoints: Vec<RouteEndpoint>,
    pub tags: Vec<String>,
    pub enabled: bool,
}

/// 路由端点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEndpoint {
    pub path: String,
    pub protocol: String,
    pub url: String,
}

/// 路由列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteListResponse {
    pub base_url: String,
    pub default_provider: String,
    pub routes: Vec<RouteInfo>,
}

/// curl 示例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurlExample {
    pub description: String,
    pub command: String,
}

impl RouteInfo {
    pub fn new(selector: String, provider_type: String) -> Self {
        Self {
            selector,
            provider_type,
            credential_count: 0,
            endpoints: Vec::new(),
            tags: Vec::new(),
            enabled: true,
        }
    }

    pub fn add_endpoint(&mut self, base_url: &str, protocol: &str) {
        let path = match protocol {
            "claude" => format!("/{}/v1/messages", self.selector),
            "openai" => format!("/{}/v1/chat/completions", self.selector),
            _ => return,
        };
        let url = format!("{base_url}{path}");
        self.endpoints.push(RouteEndpoint {
            path,
            protocol: protocol.to_string(),
            url,
        });
    }

    pub fn generate_curl_examples(&self, api_key: &str) -> Vec<CurlExample> {
        let mut examples = Vec::new();

        for endpoint in &self.endpoints {
            let (_model, body) = match endpoint.protocol.as_str() {
                "claude" => {
                    let model = match self.provider_type.as_str() {
                        "kiro" | "claude" => "claude-sonnet-4-5",
                        "gemini" => "gemini-2.5-flash",
                        "qwen" => "qwen3-coder-plus",
                        "openai" => "gpt-4",
                        _ => "claude-sonnet-4-5",
                    };
                    (
                        model,
                        format!(
                            r#"{{
  "model": "{model}",
  "max_tokens": 1024,
  "messages": [{{"role": "user", "content": "Hello!"}}]
}}"#
                        ),
                    )
                }
                "openai" => {
                    let model = match self.provider_type.as_str() {
                        "kiro" | "claude" => "claude-sonnet-4-5",
                        "gemini" => "gemini-2.5-flash",
                        "qwen" => "qwen3-coder-plus",
                        "openai" => "gpt-4",
                        _ => "claude-sonnet-4-5",
                    };
                    (
                        model,
                        format!(
                            r#"{{
  "model": "{model}",
  "messages": [{{"role": "user", "content": "Hello!"}}]
}}"#
                        ),
                    )
                }
                _ => continue,
            };

            let command = format!(
                r#"curl {} \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer {}" \
  -d '{}'"#,
                endpoint.url, api_key, body
            );

            examples.push(CurlExample {
                description: format!("{} 协议", endpoint.protocol.to_uppercase()),
                command,
            });
        }

        examples
    }
}
