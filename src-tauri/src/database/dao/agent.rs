//! Agent 会话和消息的数据访问层
//!
//! 提供 Agent 会话和消息的持久化存储功能

use crate::agent::types::{AgentMessage, AgentSession, MessageContent, ToolCall};
use rusqlite::{params, Connection};

/// 解析消息内容 JSON，支持多种格式
///
/// 支持的格式：
/// 1. Aster 格式: `[{"Text":"..."}, {"ToolRequest":...}]`
/// 2. ProxyCast 纯文本: `"string"`
/// 3. ProxyCast Parts: `[{"type":"text","text":"..."}]`
fn parse_message_content(content_json: &str) -> MessageContent {
    // 尝试解析为 Aster 格式 (Vec<AsterMessageContent>)
    if let Ok(aster_contents) = serde_json::from_str::<Vec<serde_json::Value>>(content_json) {
        let mut text_parts: Vec<String> = Vec::new();

        for item in aster_contents {
            // Aster 格式: {"Text": "..."} 或 {"ToolRequest": ...}
            if let Some(text) = item.get("Text").and_then(|v| v.as_str()) {
                text_parts.push(text.to_string());
            }
            // 也支持小写 "text" 格式
            else if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                text_parts.push(text.to_string());
            }
            // ProxyCast Parts 格式: {"type": "text", "text": "..."}
            else if item.get("type").and_then(|v| v.as_str()) == Some("text") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    text_parts.push(text.to_string());
                }
            }
            // 忽略 ToolRequest、ToolResponse 等非文本内容
        }

        if !text_parts.is_empty() {
            return MessageContent::Text(text_parts.join("\n"));
        }
    }

    // 尝试解析为纯文本字符串
    if let Ok(text) = serde_json::from_str::<String>(content_json) {
        return MessageContent::Text(text);
    }

    // 尝试直接解析为 ProxyCast MessageContent
    if let Ok(content) = serde_json::from_str::<MessageContent>(content_json) {
        return content;
    }

    // 兜底：返回原始 JSON 作为文本
    MessageContent::Text(content_json.to_string())
}

pub struct AgentDao;

impl AgentDao {
    /// 创建新会话
    pub fn create_session(
        conn: &Connection,
        session: &AgentSession,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT INTO agent_sessions (id, model, system_prompt, title, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session.id,
                session.model,
                session.system_prompt,
                session.title,
                session.created_at,
                session.updated_at,
            ],
        )?;
        Ok(())
    }

    /// 获取会话（不包含消息）
    pub fn get_session(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Option<AgentSession>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, model, system_prompt, title, created_at, updated_at
             FROM agent_sessions WHERE id = ?",
        )?;

        let mut rows = stmt.query([session_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(AgentSession {
                id: row.get(0)?,
                model: row.get(1)?,
                messages: Vec::new(), // 消息需要单独加载
                system_prompt: row.get(2)?,
                title: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取会话（包含消息）
    pub fn get_session_with_messages(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Option<AgentSession>, rusqlite::Error> {
        let mut session = match Self::get_session(conn, session_id)? {
            Some(s) => s,
            None => return Ok(None),
        };

        session.messages = Self::get_messages(conn, session_id)?;
        Ok(Some(session))
    }

    /// 获取所有会话（不包含消息）
    pub fn list_sessions(conn: &Connection) -> Result<Vec<AgentSession>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, model, system_prompt, title, created_at, updated_at
             FROM agent_sessions ORDER BY updated_at DESC",
        )?;

        let sessions = stmt.query_map([], |row| {
            Ok(AgentSession {
                id: row.get(0)?,
                model: row.get(1)?,
                messages: Vec::new(),
                system_prompt: row.get(2)?,
                title: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        sessions.collect()
    }

    /// 获取会话的消息数量
    pub fn get_message_count(
        conn: &Connection,
        session_id: &str,
    ) -> Result<usize, rusqlite::Error> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_messages WHERE session_id = ?",
            [session_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// 更新会话的 updated_at 时间
    pub fn update_session_time(
        conn: &Connection,
        session_id: &str,
        updated_at: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE agent_sessions SET updated_at = ? WHERE id = ?",
            params![updated_at, session_id],
        )?;
        Ok(())
    }

    /// 删除会话（消息会级联删除）
    pub fn delete_session(conn: &Connection, session_id: &str) -> Result<bool, rusqlite::Error> {
        let rows = conn.execute("DELETE FROM agent_sessions WHERE id = ?", [session_id])?;
        Ok(rows > 0)
    }

    /// 添加消息到会话
    pub fn add_message(
        conn: &Connection,
        session_id: &str,
        message: &AgentMessage,
    ) -> Result<(), rusqlite::Error> {
        let content_json = serde_json::to_string(&message.content)
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let tool_calls_json = message
            .tool_calls
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        conn.execute(
            "INSERT INTO agent_messages (session_id, role, content_json, timestamp, tool_calls_json, tool_call_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_id,
                message.role,
                content_json,
                message.timestamp,
                tool_calls_json,
                message.tool_call_id,
            ],
        )?;

        // 更新会话的 updated_at
        conn.execute(
            "UPDATE agent_sessions SET updated_at = ? WHERE id = ?",
            params![message.timestamp, session_id],
        )?;

        Ok(())
    }

    /// 获取会话的所有消息
    pub fn get_messages(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Vec<AgentMessage>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT role, content_json, timestamp, tool_calls_json, tool_call_id
             FROM agent_messages WHERE session_id = ? ORDER BY id ASC",
        )?;

        let messages = stmt.query_map([session_id], |row| {
            let role: String = row.get(0)?;
            let content_json: String = row.get(1)?;
            let timestamp: String = row.get(2)?;
            let tool_calls_json: Option<String> = row.get(3)?;
            let tool_call_id: Option<String> = row.get(4)?;

            // 解析 JSON - 支持多种格式
            // 1. Aster 格式: [{"Text":"..."}, {"Text":"..."}]
            // 2. ProxyCast 格式: "string" 或 [{"type":"text","text":"..."}]
            let content = parse_message_content(&content_json);

            let tool_calls: Option<Vec<ToolCall>> = tool_calls_json
                .map(|json| serde_json::from_str(&json))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(AgentMessage {
                role,
                content,
                timestamp,
                tool_calls,
                tool_call_id,
                reasoning_content: None,
            })
        })?;

        messages.collect()
    }

    /// 删除会话的所有消息
    pub fn delete_messages(conn: &Connection, session_id: &str) -> Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM agent_messages WHERE session_id = ?",
            [session_id],
        )?;
        Ok(())
    }

    /// 检查会话是否存在
    pub fn session_exists(conn: &Connection, session_id: &str) -> Result<bool, rusqlite::Error> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM agent_sessions WHERE id = ?",
            [session_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// 更新会话标题
    pub fn update_title(
        conn: &Connection,
        session_id: &str,
        title: &str,
    ) -> Result<(), rusqlite::Error> {
        conn.execute(
            "UPDATE agent_sessions SET title = ? WHERE id = ?",
            params![title, session_id],
        )?;
        Ok(())
    }

    /// 获取会话标题
    pub fn get_title(
        conn: &Connection,
        session_id: &str,
    ) -> Result<Option<String>, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT title FROM agent_sessions WHERE id = ?")?;
        let mut rows = stmt.query([session_id])?;

        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Ok(None)
        }
    }
}
