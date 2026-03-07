use std::fs;
use std::path::{Path, PathBuf};

const VIDEO_GENERATE_SKILL_NAME: &str = "video_generate";
const VIDEO_GENERATE_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/video_generate/SKILL.md");

const BROADCAST_GENERATE_SKILL_NAME: &str = "broadcast_generate";
const BROADCAST_GENERATE_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/broadcast_generate/SKILL.md");

const COVER_GENERATE_SKILL_NAME: &str = "cover_generate";
const COVER_GENERATE_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/cover_generate/SKILL.md");

const MODAL_RESOURCE_SEARCH_SKILL_NAME: &str = "modal_resource_search";
const MODAL_RESOURCE_SEARCH_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/modal_resource_search/SKILL.md");

const IMAGE_GENERATE_SKILL_NAME: &str = "image_generate";
const IMAGE_GENERATE_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/image_generate/SKILL.md");

const LIBRARY_SKILL_NAME: &str = "library";
const LIBRARY_SKILL_CONTENT: &str = include_str!("../../resources/default-skills/library/SKILL.md");

const URL_PARSE_SKILL_NAME: &str = "url_parse";
const URL_PARSE_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/url_parse/SKILL.md");

const RESEARCH_SKILL_NAME: &str = "research";
const RESEARCH_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/research/SKILL.md");

const TYPESETTING_SKILL_NAME: &str = "typesetting";
const TYPESETTING_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/typesetting/SKILL.md");

const SOCIAL_POST_WITH_COVER_SKILL_NAME: &str = "social_post_with_cover";
const SOCIAL_POST_WITH_COVER_SKILL_CONTENT: &str =
    include_str!("../../resources/default-skills/social_post_with_cover/SKILL.md");

fn default_skills() -> [(&'static str, &'static str); 10] {
    [
        (VIDEO_GENERATE_SKILL_NAME, VIDEO_GENERATE_SKILL_CONTENT),
        (
            BROADCAST_GENERATE_SKILL_NAME,
            BROADCAST_GENERATE_SKILL_CONTENT,
        ),
        (COVER_GENERATE_SKILL_NAME, COVER_GENERATE_SKILL_CONTENT),
        (
            MODAL_RESOURCE_SEARCH_SKILL_NAME,
            MODAL_RESOURCE_SEARCH_SKILL_CONTENT,
        ),
        (IMAGE_GENERATE_SKILL_NAME, IMAGE_GENERATE_SKILL_CONTENT),
        (LIBRARY_SKILL_NAME, LIBRARY_SKILL_CONTENT),
        (URL_PARSE_SKILL_NAME, URL_PARSE_SKILL_CONTENT),
        (RESEARCH_SKILL_NAME, RESEARCH_SKILL_CONTENT),
        (TYPESETTING_SKILL_NAME, TYPESETTING_SKILL_CONTENT),
        (
            SOCIAL_POST_WITH_COVER_SKILL_NAME,
            SOCIAL_POST_WITH_COVER_SKILL_CONTENT,
        ),
    ]
}

fn skills_root_from_home(home_dir: &Path) -> PathBuf {
    home_dir.join(".proxycast").join("skills")
}

fn ensure_default_local_skills_in_home(home_dir: &Path) -> Result<Vec<String>, String> {
    let skills_root = skills_root_from_home(home_dir);
    fs::create_dir_all(&skills_root)
        .map_err(|e| format!("创建技能目录失败 {}: {e}", skills_root.display()))?;

    let mut installed = Vec::new();
    for (skill_name, skill_content) in default_skills() {
        let skill_dir = skills_root.join(skill_name);
        let skill_md_path = skill_dir.join("SKILL.md");
        if skill_md_path.exists() {
            continue;
        }

        fs::create_dir_all(&skill_dir)
            .map_err(|e| format!("创建默认技能目录失败 {}: {e}", skill_dir.display()))?;
        fs::write(&skill_md_path, skill_content)
            .map_err(|e| format!("写入默认技能失败 {}: {e}", skill_md_path.display()))?;
        installed.push(skill_name.to_string());
    }
    Ok(installed)
}

pub fn ensure_default_local_skills() -> Result<Vec<String>, String> {
    let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户 Home 目录".to_string())?;
    ensure_default_local_skills_in_home(&home_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_install_default_skill_when_missing() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let installed = ensure_default_local_skills_in_home(temp.path()).expect("install");
        assert!(installed.contains(&SOCIAL_POST_WITH_COVER_SKILL_NAME.to_string()));

        let skill_md_path = temp
            .path()
            .join(".proxycast")
            .join("skills")
            .join(SOCIAL_POST_WITH_COVER_SKILL_NAME)
            .join("SKILL.md");
        assert!(skill_md_path.exists());
    }

    #[test]
    fn should_not_overwrite_existing_skill() {
        let temp = tempfile::tempdir().expect("create temp dir");
        let skill_dir = temp
            .path()
            .join(".proxycast")
            .join("skills")
            .join(SOCIAL_POST_WITH_COVER_SKILL_NAME);
        fs::create_dir_all(&skill_dir).expect("create skill dir");
        let skill_md_path = skill_dir.join("SKILL.md");
        let existing_content = "custom skill content";
        fs::write(&skill_md_path, existing_content).expect("write custom skill");

        let installed = ensure_default_local_skills_in_home(temp.path()).expect("install");
        assert!(
            !installed.contains(&SOCIAL_POST_WITH_COVER_SKILL_NAME.to_string()),
            "已存在的 skill 不应被重新安装"
        );

        let current_content = fs::read_to_string(&skill_md_path).expect("read skill");
        assert_eq!(current_content, existing_content);
    }

    #[test]
    fn should_embed_social_image_tool_contract_in_default_skill() {
        assert!(SOCIAL_POST_WITH_COVER_SKILL_CONTENT
            .contains("allowed-tools: social_generate_cover_image, search_query"));
        assert!(SOCIAL_POST_WITH_COVER_SKILL_CONTENT.contains("## 配图说明"));
        assert!(SOCIAL_POST_WITH_COVER_SKILL_CONTENT.contains("状态：{成功/失败}"));
    }

    #[test]
    fn should_embed_core_default_skills() {
        assert!(VIDEO_GENERATE_SKILL_CONTENT.contains("name: video_generate"));
        assert!(BROADCAST_GENERATE_SKILL_CONTENT.contains("name: broadcast_generate"));
        assert!(COVER_GENERATE_SKILL_CONTENT.contains("name: cover_generate"));
        assert!(MODAL_RESOURCE_SEARCH_SKILL_CONTENT.contains("name: modal_resource_search"));
        assert!(IMAGE_GENERATE_SKILL_CONTENT.contains("name: image_generate"));
        assert!(LIBRARY_SKILL_CONTENT.contains("name: library"));
        assert!(URL_PARSE_SKILL_CONTENT.contains("name: url_parse"));
        assert!(RESEARCH_SKILL_CONTENT.contains("name: research"));
        assert!(TYPESETTING_SKILL_CONTENT.contains("name: typesetting"));
    }
}
