//! OSC 序列解析器
//!
//! 解析终端输出中的 OSC（Operating System Command）序列，支持：
//! - OSC 7: 当前工作目录
//! - OSC 52: 剪贴板操作
//! - OSC 133: 命令提示符标记（Shell Integration）
//! - OSC 16162: Wave 特定命令
//!
//! ## 功能
//! - 从字节流中识别和解析 OSC 序列
//! - 支持多种 OSC 序列类型
//! - 无效序列容错处理
//!
//! ## Requirements
//! - 6.1: OSC 7 当前目录解析
//! - 6.2: OSC 52 剪贴板解析
//! - 6.3: OSC 133 命令提示符标记解析
//! - 6.4: OSC 16162 Wave 命令解析
//! - 6.7: 无效序列容错处理

use std::ops::Range;

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// OSC 序列起始标记
const OSC_START: &[u8] = b"\x1b]";
/// OSC 序列结束标记 - BEL
const OSC_END_BEL: u8 = 0x07;
/// OSC 序列结束标记 - ST (String Terminator)
/// 预留：某些终端使用 ST 而不是 BEL 作为结束标记
#[allow(dead_code)]
const OSC_END_ST: &[u8] = b"\x1b\\";

/// OSC 序列类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OSCSequence {
    /// OSC 7 - 当前工作目录
    /// 格式: OSC 7 ; file://hostname/path ST
    CurrentDirectory {
        /// 主机名（可选）
        hostname: Option<String>,
        /// 路径
        path: String,
    },

    /// OSC 52 - 剪贴板操作
    /// 格式: OSC 52 ; selection ; base64-data ST
    Clipboard {
        /// 选择类型 (c=clipboard, p=primary, s=secondary, etc.)
        selection: String,
        /// Base64 编码的数据
        data: String,
    },

    /// OSC 133 - 命令提示符标记（Shell Integration）
    /// 格式: OSC 133 ; type ST
    PromptMark {
        /// 标记类型
        mark_type: PromptMarkType,
    },

    /// OSC 16162 - Wave 特定命令
    /// 格式: OSC 16162 ; command ST
    WaveCommand {
        /// 命令内容
        command: String,
    },

    /// 未知的 OSC 序列
    Unknown {
        /// OSC 代码
        code: String,
        /// 参数
        params: String,
    },
}

/// 命令提示符标记类型（OSC 133）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptMarkType {
    /// A - 提示符开始（Prompt Start）
    PromptStart,
    /// B - 命令开始（Command Start，用户输入开始）
    CommandStart,
    /// C - 命令执行（Command Executed）
    CommandExecuted,
    /// D - 命令结束（Command Finished）
    CommandFinished,
    /// 未知标记类型
    Unknown(char),
}

impl PromptMarkType {
    /// 从字符解析标记类型
    pub fn from_char(c: char) -> Self {
        match c {
            'A' => Self::PromptStart,
            'B' => Self::CommandStart,
            'C' => Self::CommandExecuted,
            'D' => Self::CommandFinished,
            other => Self::Unknown(other),
        }
    }

    /// 转换为字符
    pub fn to_char(&self) -> char {
        match self {
            Self::PromptStart => 'A',
            Self::CommandStart => 'B',
            Self::CommandExecuted => 'C',
            Self::CommandFinished => 'D',
            Self::Unknown(c) => *c,
        }
    }
}

/// 解析结果
#[derive(Debug, Clone)]
pub struct ParsedOSC {
    /// 解析出的 OSC 序列
    pub sequence: OSCSequence,
    /// 序列在原始数据中的位置范围
    pub range: Range<usize>,
}

/// OSC 序列解析器
pub struct OSCParser;

impl OSCParser {
    /// 从字节流中解析所有 OSC 序列
    ///
    /// # 参数
    /// - `data`: 输入字节流
    ///
    /// # 返回
    /// 解析出的 OSC 序列列表，每个元素包含序列和位置范围
    ///
    /// # 示例
    /// ```
    /// use proxycast::terminal::integration::osc_parser::{OSCParser, OSCSequence};
    ///
    /// let data = b"\x1b]7;file://localhost/home/user\x07";
    /// let results = OSCParser::parse(data);
    /// assert_eq!(results.len(), 1);
    /// ```
    ///
    /// _Requirements: 6.1, 6.2, 6.3, 6.4, 6.7_
    pub fn parse(data: &[u8]) -> Vec<ParsedOSC> {
        let mut results = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            // 查找 OSC 起始标记
            if let Some(start_offset) = Self::find_osc_start(&data[pos..]) {
                let start = pos + start_offset;
                let content_start = start + OSC_START.len();

                // 查找 OSC 结束标记
                if let Some((end, terminator_len)) = Self::find_osc_end(&data[content_start..]) {
                    let content_end = content_start + end;
                    let sequence_end = content_end + terminator_len;

                    // 解析 OSC 内容
                    let content = &data[content_start..content_end];
                    if let Some(sequence) = Self::parse_osc_content(content) {
                        results.push(ParsedOSC {
                            sequence,
                            range: start..sequence_end,
                        });
                    }

                    pos = sequence_end;
                } else {
                    // 没有找到结束标记，跳过这个起始标记
                    pos = content_start;
                }
            } else {
                // 没有更多 OSC 序列
                break;
            }
        }

        results
    }

    /// 解析单个 OSC 序列
    ///
    /// # 参数
    /// - `data`: 完整的 OSC 序列（包含起始和结束标记）
    ///
    /// # 返回
    /// 解析成功返回 OSC 序列，失败返回 None
    pub fn parse_single(data: &[u8]) -> Option<OSCSequence> {
        // 检查起始标记
        if !data.starts_with(OSC_START) {
            return None;
        }

        let content_start = OSC_START.len();

        // 查找结束标记
        let (end, _) = Self::find_osc_end(&data[content_start..])?;
        let content = &data[content_start..content_start + end];

        Self::parse_osc_content(content)
    }

    /// 查找 OSC 起始标记
    fn find_osc_start(data: &[u8]) -> Option<usize> {
        data.windows(OSC_START.len()).position(|w| w == OSC_START)
    }

    /// 查找 OSC 结束标记
    ///
    /// 返回 (结束位置, 终止符长度)
    fn find_osc_end(data: &[u8]) -> Option<(usize, usize)> {
        for (i, &byte) in data.iter().enumerate() {
            // BEL 终止符
            if byte == OSC_END_BEL {
                return Some((i, 1));
            }
            // ST 终止符 (ESC \)
            if byte == 0x1b && i + 1 < data.len() && data[i + 1] == b'\\' {
                return Some((i, 2));
            }
        }
        None
    }

    /// 解析 OSC 内容
    fn parse_osc_content(content: &[u8]) -> Option<OSCSequence> {
        // 转换为字符串
        let content_str = String::from_utf8_lossy(content);

        // 分割 OSC 代码和参数
        let (code, params) = match content_str.find(';') {
            Some(pos) => (&content_str[..pos], &content_str[pos + 1..]),
            None => (content_str.as_ref(), ""),
        };

        // 根据 OSC 代码解析
        match code {
            "7" => Self::parse_osc_7(params),
            "52" => Self::parse_osc_52(params),
            "133" => Self::parse_osc_133(params),
            "16162" => Self::parse_osc_16162(params),
            _ => Some(OSCSequence::Unknown {
                code: code.to_string(),
                params: params.to_string(),
            }),
        }
    }

    /// 解析 OSC 7 - 当前工作目录
    ///
    /// 格式: file://hostname/path 或 file:///path
    ///
    /// _Requirements: 6.1_
    fn parse_osc_7(params: &str) -> Option<OSCSequence> {
        // 移除 file:// 前缀
        let path_part = params.strip_prefix("file://")?;

        // 解析主机名和路径
        let (hostname, path) = if path_part.starts_with('/') {
            // file:///path 格式（无主机名）
            (None, path_part.to_string())
        } else {
            // file://hostname/path 格式
            match path_part.find('/') {
                Some(pos) => {
                    let host = &path_part[..pos];
                    let path = &path_part[pos..];
                    (
                        if host.is_empty() {
                            None
                        } else {
                            Some(host.to_string())
                        },
                        path.to_string(),
                    )
                }
                None => {
                    // 只有主机名，没有路径
                    (Some(path_part.to_string()), "/".to_string())
                }
            }
        };

        // URL 解码路径
        let decoded_path = Self::url_decode(&path);

        Some(OSCSequence::CurrentDirectory {
            hostname,
            path: decoded_path,
        })
    }

    /// 解析 OSC 52 - 剪贴板操作
    ///
    /// 格式: selection;base64-data
    ///
    /// _Requirements: 6.2_
    fn parse_osc_52(params: &str) -> Option<OSCSequence> {
        let (selection, data) = match params.find(';') {
            Some(pos) => (&params[..pos], &params[pos + 1..]),
            None => (params, ""),
        };

        Some(OSCSequence::Clipboard {
            selection: selection.to_string(),
            data: data.to_string(),
        })
    }

    /// 解析 OSC 133 - 命令提示符标记
    ///
    /// 格式: type (A/B/C/D)
    ///
    /// _Requirements: 6.3_
    fn parse_osc_133(params: &str) -> Option<OSCSequence> {
        let mark_char = params.chars().next()?;
        let mark_type = PromptMarkType::from_char(mark_char);

        Some(OSCSequence::PromptMark { mark_type })
    }

    /// 解析 OSC 16162 - Wave 命令
    ///
    /// _Requirements: 6.4_
    fn parse_osc_16162(params: &str) -> Option<OSCSequence> {
        Some(OSCSequence::WaveCommand {
            command: params.to_string(),
        })
    }

    /// URL 解码
    fn url_decode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '%' {
                // 尝试解析两个十六进制字符
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                // 解析失败，保留原样
                result.push('%');
                result.push_str(&hex);
            } else {
                result.push(c);
            }
        }

        result
    }

    /// 从 OSC 52 数据中解码剪贴板内容
    ///
    /// # 参数
    /// - `data`: Base64 编码的数据
    ///
    /// # 返回
    /// 解码后的字符串，解码失败返回 None
    pub fn decode_clipboard_data(data: &str) -> Option<String> {
        if data == "?" {
            // 查询请求
            return None;
        }

        let decoded = BASE64.decode(data).ok()?;
        String::from_utf8(decoded).ok()
    }

    /// 编码剪贴板内容为 OSC 52 格式
    ///
    /// # 参数
    /// - `selection`: 选择类型
    /// - `content`: 要编码的内容
    ///
    /// # 返回
    /// 完整的 OSC 52 序列
    pub fn encode_clipboard(selection: &str, content: &str) -> Vec<u8> {
        let encoded = BASE64.encode(content.as_bytes());
        format!("\x1b]52;{};{}\x07", selection, encoded).into_bytes()
    }

    /// 构建 OSC 7 序列
    ///
    /// # 参数
    /// - `hostname`: 主机名（可选）
    /// - `path`: 路径
    ///
    /// # 返回
    /// 完整的 OSC 7 序列
    pub fn build_osc_7(hostname: Option<&str>, path: &str) -> Vec<u8> {
        let host = hostname.unwrap_or("");
        format!("\x1b]7;file://{}{}\x07", host, path).into_bytes()
    }

    /// 构建 OSC 133 序列
    ///
    /// # 参数
    /// - `mark_type`: 标记类型
    ///
    /// # 返回
    /// 完整的 OSC 133 序列
    pub fn build_osc_133(mark_type: PromptMarkType) -> Vec<u8> {
        format!("\x1b]133;{}\x07", mark_type.to_char()).into_bytes()
    }
}

/// 从数据流中过滤掉 OSC 序列，返回纯文本数据
///
/// # 参数
/// - `data`: 输入数据
///
/// # 返回
/// 过滤后的数据
pub fn strip_osc_sequences(data: &[u8]) -> Vec<u8> {
    let parsed = OSCParser::parse(data);
    if parsed.is_empty() {
        return data.to_vec();
    }

    let mut result = Vec::with_capacity(data.len());
    let mut last_end = 0;

    for osc in parsed {
        // 添加 OSC 序列之前的数据
        if osc.range.start > last_end {
            result.extend_from_slice(&data[last_end..osc.range.start]);
        }
        last_end = osc.range.end;
    }

    // 添加最后一个 OSC 序列之后的数据
    if last_end < data.len() {
        result.extend_from_slice(&data[last_end..]);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_osc_7_with_hostname() {
        let data = b"\x1b]7;file://localhost/home/user\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::CurrentDirectory { hostname, path } => {
                assert_eq!(hostname.as_deref(), Some("localhost"));
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected CurrentDirectory"),
        }
    }

    #[test]
    fn test_parse_osc_7_without_hostname() {
        let data = b"\x1b]7;file:///home/user\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::CurrentDirectory { hostname, path } => {
                assert!(hostname.is_none());
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected CurrentDirectory"),
        }
    }

    #[test]
    fn test_parse_osc_7_url_encoded() {
        let data = b"\x1b]7;file:///home/user/my%20folder\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::CurrentDirectory { path, .. } => {
                assert_eq!(path, "/home/user/my folder");
            }
            _ => panic!("Expected CurrentDirectory"),
        }
    }

    #[test]
    fn test_parse_osc_52() {
        let data = b"\x1b]52;c;SGVsbG8gV29ybGQ=\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::Clipboard { selection, data } => {
                assert_eq!(selection, "c");
                assert_eq!(data, "SGVsbG8gV29ybGQ=");

                // 验证解码
                let decoded = OSCParser::decode_clipboard_data(data);
                assert_eq!(decoded, Some("Hello World".to_string()));
            }
            _ => panic!("Expected Clipboard"),
        }
    }

    #[test]
    fn test_parse_osc_133() {
        // 测试所有标记类型
        let test_cases = [
            (b"\x1b]133;A\x07".as_slice(), PromptMarkType::PromptStart),
            (b"\x1b]133;B\x07".as_slice(), PromptMarkType::CommandStart),
            (
                b"\x1b]133;C\x07".as_slice(),
                PromptMarkType::CommandExecuted,
            ),
            (
                b"\x1b]133;D\x07".as_slice(),
                PromptMarkType::CommandFinished,
            ),
        ];

        for (data, expected_type) in test_cases {
            let results = OSCParser::parse(data);
            assert_eq!(results.len(), 1);
            match &results[0].sequence {
                OSCSequence::PromptMark { mark_type } => {
                    assert_eq!(*mark_type, expected_type);
                }
                _ => panic!("Expected PromptMark"),
            }
        }
    }

    #[test]
    fn test_parse_osc_16162() {
        let data = b"\x1b]16162;setcwd /home/user\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::WaveCommand { command } => {
                assert_eq!(command, "setcwd /home/user");
            }
            _ => panic!("Expected WaveCommand"),
        }
    }

    #[test]
    fn test_parse_multiple_osc() {
        let data = b"Hello\x1b]7;file:///home\x07World\x1b]133;A\x07End";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 2);
        assert!(matches!(
            &results[0].sequence,
            OSCSequence::CurrentDirectory { .. }
        ));
        assert!(matches!(
            &results[1].sequence,
            OSCSequence::PromptMark { .. }
        ));
    }

    #[test]
    fn test_parse_osc_with_st_terminator() {
        let data = b"\x1b]7;file:///home/user\x1b\\";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::CurrentDirectory { path, .. } => {
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected CurrentDirectory"),
        }
    }

    #[test]
    fn test_parse_unknown_osc() {
        let data = b"\x1b]999;some params\x07";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::Unknown { code, params } => {
                assert_eq!(code, "999");
                assert_eq!(params, "some params");
            }
            _ => panic!("Expected Unknown"),
        }
    }

    #[test]
    fn test_parse_no_osc() {
        let data = b"Hello World";
        let results = OSCParser::parse(data);
        assert!(results.is_empty());
    }

    #[test]
    fn test_strip_osc_sequences() {
        let data = b"Hello\x1b]7;file:///home\x07World";
        let stripped = strip_osc_sequences(data);
        assert_eq!(stripped, b"HelloWorld");
    }

    #[test]
    fn test_encode_clipboard() {
        let encoded = OSCParser::encode_clipboard("c", "Hello");
        let results = OSCParser::parse(&encoded);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::Clipboard { selection, data } => {
                assert_eq!(selection, "c");
                let decoded = OSCParser::decode_clipboard_data(data);
                assert_eq!(decoded, Some("Hello".to_string()));
            }
            _ => panic!("Expected Clipboard"),
        }
    }

    #[test]
    fn test_build_osc_7() {
        let osc = OSCParser::build_osc_7(Some("localhost"), "/home/user");
        let results = OSCParser::parse(&osc);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::CurrentDirectory { hostname, path } => {
                assert_eq!(hostname.as_deref(), Some("localhost"));
                assert_eq!(path, "/home/user");
            }
            _ => panic!("Expected CurrentDirectory"),
        }
    }

    #[test]
    fn test_build_osc_133() {
        let osc = OSCParser::build_osc_133(PromptMarkType::PromptStart);
        let results = OSCParser::parse(&osc);

        assert_eq!(results.len(), 1);
        match &results[0].sequence {
            OSCSequence::PromptMark { mark_type } => {
                assert_eq!(*mark_type, PromptMarkType::PromptStart);
            }
            _ => panic!("Expected PromptMark"),
        }
    }

    #[test]
    fn test_prompt_mark_type_roundtrip() {
        let types = [
            PromptMarkType::PromptStart,
            PromptMarkType::CommandStart,
            PromptMarkType::CommandExecuted,
            PromptMarkType::CommandFinished,
            PromptMarkType::Unknown('X'),
        ];

        for mark_type in types {
            let c = mark_type.to_char();
            let parsed = PromptMarkType::from_char(c);
            assert_eq!(mark_type, parsed);
        }
    }

    #[test]
    fn test_parse_range() {
        let data = b"ABC\x1b]7;file:///home\x07XYZ";
        let results = OSCParser::parse(data);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].range.start, 3);
        // OSC 序列: \x1b]7;file:///home\x07
        // = ESC(1) + ](1) + "7;file:///home"(14) + BEL(1) = 17 bytes
        // 所以 end = 3 + 17 = 20
        assert_eq!(results[0].range.end, 20);
    }
}
