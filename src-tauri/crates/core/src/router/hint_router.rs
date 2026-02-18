//! 提示路由器
//!
//! 支持通过消息前缀提示（hint）将请求路由到不同的 Provider 和模型。
//!
//! 提示格式：`[hint] 消息内容`
//! 例如：`[reasoning] 请分析这段代码的复杂度`
//!       `[fast] 翻译这句话`
//!       `[code] 实现一个排序算法`

use crate::ProviderType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 提示路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintRouterConfig {
    /// 是否启用提示路由
    #[serde(default)]
    pub enabled: bool,
    /// 提示路由规则
    #[serde(default)]
    pub routes: Vec<HintRouteEntry>,
}

impl Default for HintRouterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            routes: Vec::new(),
        }
    }
}

/// 单条提示路由配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HintRouteEntry {
    /// 提示关键词（如 "reasoning", "fast", "code"）
    pub hint: String,
    /// 目标 Provider
    pub provider: ProviderType,
    /// 目标模型
    pub model: String,
}

/// 已解析的提示路由
#[derive(Debug, Clone, PartialEq)]
pub struct HintRoute {
    /// 提示关键词
    pub hint: String,
    /// 目标 Provider
    pub provider: ProviderType,
    /// 目标模型
    pub model: String,
}

/// 提示匹配结果
#[derive(Debug, Clone, PartialEq)]
pub struct HintMatch {
    /// 匹配到的路由
    pub route: HintRoute,
    /// 去除提示前缀后的消息内容
    pub stripped_message: String,
}

/// 提示路由器
#[derive(Debug, Clone)]
pub struct HintRouter {
    enabled: bool,
    /// hint 关键词 -> 路由（小写匹配）
    routes: HashMap<String, HintRoute>,
}

impl HintRouter {
    /// 从配置创建提示路由器
    pub fn from_config(config: &HintRouterConfig) -> Self {
        let mut routes = HashMap::new();

        if config.enabled {
            for entry in &config.routes {
                let key = entry.hint.to_lowercase();
                routes.insert(
                    key,
                    HintRoute {
                        hint: entry.hint.clone(),
                        provider: entry.provider,
                        model: entry.model.clone(),
                    },
                );
            }
        }

        Self {
            enabled: config.enabled,
            routes,
        }
    }

    /// 是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取已注册的路由数量
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// 根据提示关键词查找路由
    pub fn route_by_hint(&self, hint: &str) -> Option<&HintRoute> {
        if !self.enabled {
            return None;
        }
        self.routes.get(&hint.to_lowercase())
    }

    /// 从消息中提取提示并匹配路由
    ///
    /// 支持格式：`[hint] 消息内容` 或 `[hint]消息内容`
    /// 提示匹配不区分大小写
    pub fn match_message(&self, message: &str) -> Option<HintMatch> {
        if !self.enabled {
            return None;
        }

        let trimmed = message.trim_start();
        if !trimmed.starts_with('[') {
            return None;
        }

        let close_bracket = trimmed.find(']')?;
        let hint = trimmed[1..close_bracket].trim();

        if hint.is_empty() {
            return None;
        }

        let route = self.routes.get(&hint.to_lowercase())?;

        // 提取去除前缀后的消息
        let rest = trimmed[close_bracket + 1..].trim_start();

        Some(HintMatch {
            route: route.clone(),
            stripped_message: rest.to_string(),
        })
    }
}

impl Default for HintRouter {
    fn default() -> Self {
        Self {
            enabled: false,
            routes: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HintRouterConfig {
        HintRouterConfig {
            enabled: true,
            routes: vec![
                HintRouteEntry {
                    hint: "reasoning".to_string(),
                    provider: ProviderType::Kiro,
                    model: "claude-sonnet-4-5-20250514".to_string(),
                },
                HintRouteEntry {
                    hint: "fast".to_string(),
                    provider: ProviderType::Gemini,
                    model: "gemini-2.0-flash".to_string(),
                },
                HintRouteEntry {
                    hint: "code".to_string(),
                    provider: ProviderType::Kiro,
                    model: "claude-sonnet-4-5-20250514".to_string(),
                },
            ],
        }
    }

    #[test]
    fn test_disabled_router() {
        let config = HintRouterConfig::default();
        let router = HintRouter::from_config(&config);
        assert!(!router.is_enabled());
        assert_eq!(router.route_count(), 0);
        assert!(router.route_by_hint("reasoning").is_none());
        assert!(router.match_message("[reasoning] test").is_none());
    }

    #[test]
    fn test_route_by_hint() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.is_enabled());
        assert_eq!(router.route_count(), 3);

        let route = router.route_by_hint("reasoning").unwrap();
        assert_eq!(route.provider, ProviderType::Kiro);

        let route = router.route_by_hint("fast").unwrap();
        assert_eq!(route.provider, ProviderType::Gemini);

        assert!(router.route_by_hint("unknown").is_none());
    }

    #[test]
    fn test_case_insensitive_hint() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.route_by_hint("Reasoning").is_some());
        assert!(router.route_by_hint("FAST").is_some());
        assert!(router.route_by_hint("Code").is_some());
    }

    #[test]
    fn test_match_message_basic() {
        let router = HintRouter::from_config(&test_config());

        let m = router.match_message("[reasoning] 请分析这段代码").unwrap();
        assert_eq!(m.route.hint, "reasoning");
        assert_eq!(m.route.provider, ProviderType::Kiro);
        assert_eq!(m.stripped_message, "请分析这段代码");
    }

    #[test]
    fn test_match_message_no_space() {
        let router = HintRouter::from_config(&test_config());

        let m = router.match_message("[fast]翻译这句话").unwrap();
        assert_eq!(m.route.hint, "fast");
        assert_eq!(m.stripped_message, "翻译这句话");
    }

    #[test]
    fn test_match_message_case_insensitive() {
        let router = HintRouter::from_config(&test_config());

        let m = router.match_message("[REASONING] test").unwrap();
        assert_eq!(m.route.hint, "reasoning");
    }

    #[test]
    fn test_match_message_leading_whitespace() {
        let router = HintRouter::from_config(&test_config());

        let m = router.match_message("  [fast] hello").unwrap();
        assert_eq!(m.route.hint, "fast");
        assert_eq!(m.stripped_message, "hello");
    }

    #[test]
    fn test_match_message_no_hint() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.match_message("普通消息").is_none());
        assert!(router.match_message("").is_none());
    }

    #[test]
    fn test_match_message_unknown_hint() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.match_message("[unknown] test").is_none());
    }

    #[test]
    fn test_match_message_empty_hint() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.match_message("[] test").is_none());
    }

    #[test]
    fn test_match_message_no_closing_bracket() {
        let router = HintRouter::from_config(&test_config());
        assert!(router.match_message("[reasoning test").is_none());
    }

    #[test]
    fn test_default_config() {
        let config = HintRouterConfig::default();
        assert!(!config.enabled);
        assert!(config.routes.is_empty());
    }
}
