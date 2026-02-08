//! Skill 定义加载器
//!
//! 负责从 `~/.proxycast/skills/<skill>/SKILL.md` 加载并解析 Skill 定义。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Skill 前置元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
    #[serde(rename = "argument-hint")]
    pub argument_hint: Option<String>,
    #[serde(rename = "when-to-use")]
    pub when_to_use: Option<String>,
    pub version: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    #[serde(rename = "disable-model-invocation")]
    pub disable_model_invocation: Option<String>,
    #[serde(rename = "execution-mode")]
    pub execution_mode: Option<String>,
}

/// 内部 Skill 定义（用于加载和执行）
#[derive(Debug, Clone)]
pub struct LoadedSkillDefinition {
    pub skill_name: String,
    pub display_name: String,
    pub description: String,
    pub markdown_content: String,
    pub allowed_tools: Option<Vec<String>>,
    pub argument_hint: Option<String>,
    pub when_to_use: Option<String>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub disable_model_invocation: bool,
    pub execution_mode: String,
}

/// 解析 Skill 文件的 frontmatter
pub fn parse_skill_frontmatter(content: &str) -> (SkillFrontmatter, String) {
    let regex = regex::Regex::new(r"^---\s*\n([\s\S]*?)---\s*\n?").unwrap();

    if let Some(captures) = regex.captures(content) {
        let frontmatter_text = captures.get(1).map(|m| m.as_str()).unwrap_or("");
        let body_start = captures.get(0).map(|m| m.end()).unwrap_or(0);
        let body = content.get(body_start..).unwrap_or("").to_string();

        let mut frontmatter = SkillFrontmatter::default();

        for line in frontmatter_text.lines() {
            if let Some(colon_idx) = line.find(':') {
                let key = line.get(..colon_idx).unwrap_or("").trim();
                let value = line.get(colon_idx + 1..).unwrap_or("").trim();
                let clean_value = value
                    .trim_start_matches('"')
                    .trim_end_matches('"')
                    .trim_start_matches('\'')
                    .trim_end_matches('\'')
                    .to_string();

                match key {
                    "name" => frontmatter.name = Some(clean_value),
                    "description" => frontmatter.description = Some(clean_value),
                    "allowed-tools" => frontmatter.allowed_tools = Some(clean_value),
                    "argument-hint" => frontmatter.argument_hint = Some(clean_value),
                    "when-to-use" | "when_to_use" => frontmatter.when_to_use = Some(clean_value),
                    "version" => frontmatter.version = Some(clean_value),
                    "model" => frontmatter.model = Some(clean_value),
                    "provider" => frontmatter.provider = Some(clean_value),
                    "disable-model-invocation" => {
                        frontmatter.disable_model_invocation = Some(clean_value)
                    }
                    "execution-mode" => frontmatter.execution_mode = Some(clean_value),
                    _ => {}
                }
            }
        }

        (frontmatter, body)
    } else {
        (SkillFrontmatter::default(), content.to_string())
    }
}

/// 解析 allowed-tools 字段
pub fn parse_allowed_tools(value: Option<&str>) -> Option<Vec<String>> {
    value.and_then(|v| {
        if v.is_empty() {
            return None;
        }
        if v.contains(',') {
            Some(
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            )
        } else {
            Some(vec![v.trim().to_string()])
        }
    })
}

/// 解析布尔值字段
pub fn parse_boolean(value: Option<&str>, default: bool) -> bool {
    value
        .map(|v| {
            let lower = v.to_lowercase();
            matches!(lower.as_str(), "true" | "1" | "yes")
        })
        .unwrap_or(default)
}

/// 从文件加载 Skill 定义
pub fn load_skill_from_file(
    skill_name: &str,
    file_path: &Path,
) -> Result<LoadedSkillDefinition, String> {
    let content =
        std::fs::read_to_string(file_path).map_err(|e| format!("读取 Skill 文件失败: {}", e))?;

    let (frontmatter, markdown_content) = parse_skill_frontmatter(&content);

    let display_name = frontmatter
        .name
        .clone()
        .unwrap_or_else(|| skill_name.to_string());
    let description = frontmatter.description.clone().unwrap_or_default();
    let allowed_tools = parse_allowed_tools(frontmatter.allowed_tools.as_deref());
    let disable_model_invocation =
        parse_boolean(frontmatter.disable_model_invocation.as_deref(), false);
    let execution_mode = frontmatter
        .execution_mode
        .clone()
        .unwrap_or_else(|| "prompt".to_string());

    Ok(LoadedSkillDefinition {
        skill_name: skill_name.to_string(),
        display_name,
        description,
        markdown_content,
        allowed_tools,
        argument_hint: frontmatter.argument_hint,
        when_to_use: frontmatter.when_to_use,
        model: frontmatter.model,
        provider: frontmatter.provider,
        disable_model_invocation,
        execution_mode,
    })
}

/// 获取 ProxyCast Skills 目录
pub fn get_proxycast_skills_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".proxycast").join("skills"))
}

/// 从目录加载所有 Skills
pub fn load_skills_from_directory(dir_path: &Path) -> Vec<LoadedSkillDefinition> {
    let mut results = Vec::new();

    if !dir_path.exists() {
        return results;
    }

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let skill_file = path.join("SKILL.md");
            if skill_file.exists() {
                let skill_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                if let Ok(skill) = load_skill_from_file(&skill_name, &skill_file) {
                    results.push(skill);
                }
            }
        }
    }

    results
}

/// 根据名称查找 Skill
pub fn find_skill_by_name(skill_name: &str) -> Result<LoadedSkillDefinition, String> {
    let skills_dir =
        get_proxycast_skills_dir().ok_or_else(|| "无法获取 Skills 目录".to_string())?;

    let skill_path = skills_dir.join(skill_name);
    let skill_file = skill_path.join("SKILL.md");

    if !skill_file.exists() {
        return Err(format!("Skill 不存在: {}", skill_name));
    }

    load_skill_from_file(skill_name, &skill_file)
}
