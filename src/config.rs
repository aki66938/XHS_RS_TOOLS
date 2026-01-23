//! 配置模块 (Configuration Module)
//!
//! 统一管理应用配置，支持环境变量覆盖

use std::sync::LazyLock;

/// Agent 配置
pub struct AgentConfig {
    /// Agent 服务 URL
    pub url: String,
    /// 是否为容器模式（检测到 XHS_AGENT_URL 环境变量）
    pub is_container_mode: bool,
}

impl AgentConfig {
    fn from_env() -> Self {
        match std::env::var("XHS_AGENT_URL") {
            Ok(url) => Self {
                url,
                is_container_mode: true,
            },
            Err(_) => Self {
                url: "http://127.0.0.1:8765".to_string(),
                is_container_mode: false,
            },
        }
    }
}

/// 全局 Agent 配置实例
pub static AGENT_CONFIG: LazyLock<AgentConfig> = LazyLock::new(AgentConfig::from_env);

/// 获取 Agent URL
pub fn get_agent_url() -> &'static str {
    &AGENT_CONFIG.url
}

/// 检查是否为容器模式
pub fn is_container_mode() -> bool {
    AGENT_CONFIG.is_container_mode
}
