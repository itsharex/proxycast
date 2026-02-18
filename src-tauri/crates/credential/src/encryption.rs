//! ChaCha20-Poly1305 AEAD 加密模块
//!
//! 提供凭证加密/解密功能：
//! - ChaCha20-Poly1305 认证加密（防篡改）
//! - 随机 nonce（每次加密生成新的 12 字节 nonce）
//! - 密钥派生（SHA-256）
//! - 格式：enc2:base64(nonce || ciphertext || tag)

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use sha2::{Digest, Sha256};

/// 加密前缀标识
const ENCRYPTED_PREFIX: &str = "enc2:";

/// Nonce 长度（12 字节）
const NONCE_SIZE: usize = 12;

/// 加密器
pub struct Encryptor {
    cipher: ChaCha20Poly1305,
}

impl Encryptor {
    /// 从密码/密钥创建加密器
    ///
    /// 使用 SHA-256 将任意长度的密钥派生为 256-bit 密钥
    pub fn new(key: &str) -> Self {
        let derived_key = Self::derive_key(key);
        let cipher = ChaCha20Poly1305::new(&derived_key.into());
        Self { cipher }
    }

    /// 从原始 32 字节密钥创建加密器
    pub fn from_raw_key(key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new(key.into());
        Self { cipher }
    }

    /// 使用 SHA-256 派生 256-bit 密钥
    fn derive_key(password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// 加密明文
    ///
    /// 返回格式：enc2:base64(nonce || ciphertext)
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        use chacha20poly1305::aead::AeadCore;

        // 生成随机 nonce
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| EncryptionError::EncryptionFailed)?;

        // 组合 nonce + ciphertext
        let mut combined = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        combined.extend_from_slice(&nonce);
        combined.extend_from_slice(&ciphertext);

        // Base64 编码并添加前缀
        Ok(format!("{}{}", ENCRYPTED_PREFIX, BASE64.encode(&combined)))
    }

    /// 解密密文
    ///
    /// 输入格式：enc2:base64(nonce || ciphertext)
    pub fn decrypt(&self, encrypted: &str) -> Result<String, EncryptionError> {
        // 检查前缀
        let encoded = encrypted
            .strip_prefix(ENCRYPTED_PREFIX)
            .ok_or(EncryptionError::InvalidFormat)?;

        // Base64 解码
        let combined = BASE64
            .decode(encoded)
            .map_err(|_| EncryptionError::InvalidBase64)?;

        // 分离 nonce 和 ciphertext
        if combined.len() < NONCE_SIZE {
            return Err(EncryptionError::InvalidFormat);
        }

        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| EncryptionError::DecryptionFailed)?;

        String::from_utf8(plaintext).map_err(|_| EncryptionError::InvalidUtf8)
    }

    /// 检查文本是否已加密
    pub fn is_encrypted(text: &str) -> bool {
        text.starts_with(ENCRYPTED_PREFIX)
    }

    /// 加密（如果尚未加密）
    pub fn encrypt_if_needed(&self, text: &str) -> Result<String, EncryptionError> {
        if Self::is_encrypted(text) {
            Ok(text.to_string())
        } else {
            self.encrypt(text)
        }
    }

    /// 解密（如果已加密）
    pub fn decrypt_if_needed(&self, text: &str) -> Result<String, EncryptionError> {
        if Self::is_encrypted(text) {
            self.decrypt(text)
        } else {
            Ok(text.to_string())
        }
    }
}

/// 加密错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncryptionError {
    /// 加密失败
    EncryptionFailed,
    /// 解密失败（密钥错误或数据被篡改）
    DecryptionFailed,
    /// 无效的格式（缺少 enc2: 前缀）
    InvalidFormat,
    /// 无效的 Base64 编码
    InvalidBase64,
    /// 无效的 UTF-8
    InvalidUtf8,
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EncryptionFailed => write!(f, "加密失败"),
            Self::DecryptionFailed => write!(f, "解密失败：密钥错误或数据被篡改"),
            Self::InvalidFormat => write!(f, "无效的加密格式"),
            Self::InvalidBase64 => write!(f, "无效的 Base64 编码"),
            Self::InvalidUtf8 => write!(f, "无效的 UTF-8 编码"),
        }
    }
}

impl std::error::Error for EncryptionError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let enc = Encryptor::new("test-password");
        let plaintext = "sk-abc123-secret-api-key";
        let encrypted = enc.encrypt(plaintext).unwrap();
        assert!(encrypted.starts_with(ENCRYPTED_PREFIX));
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces() {
        let enc = Encryptor::new("test-password");
        let plaintext = "same-plaintext";
        let encrypted1 = enc.encrypt(plaintext).unwrap();
        let encrypted2 = enc.encrypt(plaintext).unwrap();
        assert_ne!(encrypted1, encrypted2);
        // 两者都能正确解密
        assert_eq!(enc.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(enc.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let enc1 = Encryptor::new("correct-password");
        let enc2 = Encryptor::new("wrong-password");
        let encrypted = enc1.encrypt("secret").unwrap();
        assert_eq!(enc2.decrypt(&encrypted), Err(EncryptionError::DecryptionFailed));
    }

    #[test]
    fn test_is_encrypted() {
        assert!(Encryptor::is_encrypted("enc2:abc123"));
        assert!(!Encryptor::is_encrypted("plain-text"));
        assert!(!Encryptor::is_encrypted("enc1:old-format"));
        assert!(!Encryptor::is_encrypted(""));
    }

    #[test]
    fn test_encrypt_if_needed_already_encrypted() {
        let enc = Encryptor::new("key");
        let already = "enc2:already-encrypted-data";
        let result = enc.encrypt_if_needed(already).unwrap();
        assert_eq!(result, already);
    }

    #[test]
    fn test_decrypt_if_needed_not_encrypted() {
        let enc = Encryptor::new("key");
        let plain = "not-encrypted";
        let result = enc.decrypt_if_needed(plain).unwrap();
        assert_eq!(result, plain);
    }

    #[test]
    fn test_invalid_format() {
        let enc = Encryptor::new("key");
        assert_eq!(enc.decrypt("no-prefix"), Err(EncryptionError::InvalidFormat));
    }

    #[test]
    fn test_invalid_base64() {
        let enc = Encryptor::new("key");
        assert_eq!(enc.decrypt("enc2:!!!invalid-base64!!!"), Err(EncryptionError::InvalidBase64));
    }

    #[test]
    fn test_tampered_data() {
        let enc = Encryptor::new("key");
        let encrypted = enc.encrypt("secret").unwrap();
        // 篡改密文中的一个字符
        let encoded = encrypted.strip_prefix(ENCRYPTED_PREFIX).unwrap();
        let mut bytes = BASE64.decode(encoded).unwrap();
        if let Some(last) = bytes.last_mut() {
            *last ^= 0xFF;
        }
        let tampered = format!("{}{}", ENCRYPTED_PREFIX, BASE64.encode(&bytes));
        assert_eq!(enc.decrypt(&tampered), Err(EncryptionError::DecryptionFailed));
    }

    #[test]
    fn test_empty_string() {
        let enc = Encryptor::new("key");
        let encrypted = enc.encrypt("").unwrap();
        assert_eq!(enc.decrypt(&encrypted).unwrap(), "");
    }

    #[test]
    fn test_unicode_content() {
        let enc = Encryptor::new("密钥");
        let plaintext = "你好世界 🌍 こんにちは";
        let encrypted = enc.encrypt(plaintext).unwrap();
        assert_eq!(enc.decrypt(&encrypted).unwrap(), plaintext);
    }

    #[test]
    fn test_from_raw_key() {
        let raw_key: [u8; 32] = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ];
        let enc = Encryptor::from_raw_key(&raw_key);
        let plaintext = "raw-key-test";
        let encrypted = enc.encrypt(plaintext).unwrap();
        assert_eq!(enc.decrypt(&encrypted).unwrap(), plaintext);
    }
}
