//! 辅助函数
//!
//! 包含通用工具函数。

use crate::config;

/// 生成安全的 API Key
pub fn generate_api_key() -> String {
    config::generate_secure_api_key()
}

/// 检查是否为回环地址
pub fn is_loopback_host(host: &str) -> bool {
    if host == "localhost" {
        return true;
    }
    match host.parse::<std::net::IpAddr>() {
        Ok(addr) => addr.is_loopback(),
        Err(_) => false,
    }
}

/// 掩码敏感 Token
pub fn mask_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    if chars.len() <= 12 {
        "****".to_string()
    } else {
        let prefix: String = chars[..6].iter().collect();
        let suffix: String = chars[chars.len() - 4..].iter().collect();
        format!("{prefix}****{suffix}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_loopback_host() {
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("::1"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("192.168.1.1"));
    }

    #[test]
    fn test_mask_token() {
        assert_eq!(mask_token("short"), "****");
        assert_eq!(mask_token("abcdefghijklmnop"), "abcdef****mnop");
    }
}
