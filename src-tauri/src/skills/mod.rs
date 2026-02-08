//! Skills 集成模块
//!
//! trait 定义和纯逻辑已迁移到 proxycast-skills crate，
//! 本模块保留 Tauri 相关的实现。

mod execution_callback;
mod llm_provider;

// 从 proxycast-skills crate re-export
pub use proxycast_skills::{
    events, ExecutionCallback, ExecutionCompletePayload, LlmProvider, SkillError,
    StepCompletePayload, StepErrorPayload, StepStartPayload,
};
pub use proxycast_skills::{
    find_skill_by_name, get_proxycast_skills_dir, load_skill_from_file, load_skills_from_directory,
    parse_allowed_tools, parse_boolean, parse_skill_frontmatter,
};

// Tauri 实现（留在主 crate）
pub use execution_callback::TauriExecutionCallback;
pub use llm_provider::ProxyCastLlmProvider;
