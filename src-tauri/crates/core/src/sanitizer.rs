//! 凭证清理模块
//!
//! 使用正则表达式从文本中清理敏感信息（API 密钥、token、密码等）

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// 清理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizeConfig {
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 替换文本
    #[serde(default = "default_replacement")]
    pub replacement: String,
    /// 用户自定义正则模式
    #[serde(default)]
    pub custom_patterns: Vec<String>,
}

fn default_enabled() -> bool {
    true
}
fn default_replacement() -> String {
    "[REDACTED]".to_string()
}

impl Default for SanitizeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            replacement: default_replacement(),
            custom_patterns: Vec::new(),
        }
    }
}

/// 凭证清理器
pub struct CredentialSanitizer {
    config: SanitizeConfig,
    custom_regexes: Vec<Regex>,
}

/// 内置的敏感信息正则模式
fn builtin_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let patterns = [
            // OpenAI / Anthropic API 密钥
            r"sk-[a-zA-Z0-9_-]{20,}",
            // Anthropic 密钥
            r"sk-ant-[a-zA-Z0-9_-]{20,}",
            // AWS Access Key
            r"AKIA[0-9A-Z]{16}",
            // Groq 密钥
            r"gsk_[a-zA-Z0-9]{20,}",
            // Google API 密钥
            r"AIza[0-9A-Za-z_-]{35}",
            // Bearer token
            r"Bearer\s+[a-zA-Z0-9_\-.]+",
            // 通用 key=value 模式
            r"(?i)(api[_-]?key|secret[_-]?key|access[_-]?token|auth[_-]?token|password|passwd|secret)\s*[=:]\s*\S+",
            // GitHub token
            r"gh[pousr]_[A-Za-z0-9_]{36,}",
            // 通用长 hex/base64 token（40+ 字符）
            r#"(?i)(token|key|secret|credential)\s*[=:]\s*['"]?[a-zA-Z0-9+/=_-]{40,}['"]?"#,
        ];
        patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect()
    })
}

impl CredentialSanitizer {
    /// 创建新的清理器
    pub fn new(config: SanitizeConfig) -> Self {
        let custom_regexes = config
            .custom_patterns
            .iter()
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        Self {
            config,
            custom_regexes,
        }
    }

    /// 创建默认清理器
    pub fn with_defaults() -> Self {
        Self::new(SanitizeConfig::default())
    }

    /// 清理文本中的敏感信息
    pub fn sanitize(&self, text: &str) -> String {
        if !self.config.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();
        let replacement = &self.config.replacement;

        // 应用内置模式
        for pattern in builtin_patterns() {
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
        }

        // 应用自定义模式
        for pattern in &self.custom_regexes {
            result = pattern.replace_all(&result, replacement.as_str()).to_string();
        }

        result
    }

    /// 检查文本是否包含敏感信息
    pub fn contains_sensitive(&self, text: &str) -> bool {
        for pattern in builtin_patterns() {
            if pattern.is_match(text) {
                return true;
            }
        }
        for pattern in &self.custom_regexes {
            if pattern.is_match(text) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_openai_key() {
        let s = CredentialSanitizer::with_defaults();
        let input = "my key is sk-abc123def456ghi789jkl012mno";
        let result = s.sanitize(input);
        assert!(!result.contains("sk-abc123"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_anthropic_key() {
        let s = CredentialSanitizer::with_defaults();
        let input = "key: sk-ant-api03-abcdefghijklmnopqrstuvwxyz";
        let result = s.sanitize(input);
        assert!(!result.contains("sk-ant-"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_aws_key() {
        let s = CredentialSanitizer::with_defaults();
        let input = "aws_access_key_id = AKIAIOSFODNN7EXAMPLE";
        let result = s.sanitize(input);
        assert!(!result.contains("AKIAIOSFODNN7EXAMPLE"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let s = CredentialSanitizer::with_defaults();
        let input = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.test";
        let result = s.sanitize(input);
        assert!(!result.contains("eyJhbGci"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_key_value_pairs() {
        let s = CredentialSanitizer::with_defaults();
        let input = "api_key=super_secret_value_123";
        let result = s.sanitize(input);
        assert!(!result.contains("super_secret_value_123"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_github_token() {
        let s = CredentialSanitizer::with_defaults();
        let input = "token: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn";
        let result = s.sanitize(input);
        assert!(!result.contains("ghp_"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_disabled_returns_original() {
        let config = SanitizeConfig {
            enabled: false,
            ..Default::default()
        };
        let s = CredentialSanitizer::new(config);
        let input = "sk-abc123def456ghi789jkl012mno";
        assert_eq!(s.sanitize(input), input);
    }

    #[test]
    fn test_custom_patterns() {
        let config = SanitizeConfig {
            custom_patterns: vec![r"my-custom-\d+".to_string()],
            ..Default::default()
        };
        let s = CredentialSanitizer::new(config);
        let input = "value is my-custom-12345 here";
        let result = s.sanitize(input);
        assert!(!result.contains("my-custom-12345"));
        assert!(result.contains("[REDACTED]"));
    }

    #[test]
    fn test_contains_sensitive() {
        let s = CredentialSanitizer::with_defaults();
        assert!(s.contains_sensitive("sk-abc123def456ghi789jkl012mno"));
        assert!(s.contains_sensitive("AKIAIOSFODNN7EXAMPLE"));
        assert!(!s.contains_sensitive("hello world"));
    }

    #[test]
    fn test_no_false_positives() {
        let s = CredentialSanitizer::with_defaults();
        let normal_texts = [
            "Hello, this is a normal message.",
            "The temperature is 72 degrees.",
            "Please check the documentation at docs.rs",
            "User ID: 12345",
            "sk-short",
        ];
        for text in &normal_texts {
            assert_eq!(s.sanitize(text), *text, "False positive on: {text}");
        }
    }
}
