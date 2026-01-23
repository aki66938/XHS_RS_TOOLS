//! Python Agent 进程管理模块
//!
//! 自动管理 Python Signature Agent 的生命周期：
//! - Rust 服务启动时启动 Agent
//! - Rust 服务退出时清理 Agent

use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::path::PathBuf;
use tracing::{info, warn};

/// Agent 进程管理器
pub struct AgentManager {
    process: Mutex<Option<Child>>,
}

impl AgentManager {
    /// 创建新的 Agent 管理器
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    /// 启动 Python Agent Server
    /// 
    /// 在容器模式下（检测到 XHS_AGENT_URL 环境变量），跳过子进程启动
    pub fn start(&self) -> anyhow::Result<()> {
        // 容器模式：跳过子进程管理
        if crate::config::is_container_mode() {
            info!("[AgentManager] Container mode detected (XHS_AGENT_URL set), skipping subprocess management");
            info!("[AgentManager] Agent URL: {}", crate::config::get_agent_url());
            return Ok(());
        }
        
        let script_path = self.get_agent_script_path()?;
        
        info!("[AgentManager] Starting Python Agent: {:?}", script_path);
        
        let child = Command::new("python")
            .arg("-m")
            .arg("uvicorn")
            .arg("scripts.agent_server:app")
            .arg("--host")
            .arg("127.0.0.1")
            .arg("--port")
            .arg("8765")
            .current_dir(self.get_project_root()?)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        
        let pid = child.id();
        info!("[AgentManager] Agent started with PID: {}", pid);
        
        *self.process.lock().unwrap() = Some(child);
        
        // 等待 Agent 启动
        std::thread::sleep(std::time::Duration::from_millis(1500));
        
        Ok(())
    }

    /// 停止 Agent 进程
    pub fn stop(&self) {
        let mut guard = self.process.lock().unwrap();
        if let Some(mut child) = guard.take() {
            info!("[AgentManager] Stopping Agent (PID: {})...", child.id());
            match child.kill() {
                Ok(_) => info!("[AgentManager] Agent stopped"),
                Err(e) => warn!("[AgentManager] Failed to kill Agent: {}", e),
            }
        }
    }

    /// 检查 Agent 是否正在运行
    pub fn is_running(&self) -> bool {
        let mut guard = self.process.lock().unwrap();
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => true,  // 仍在运行
                Ok(Some(_)) => false,  // 已退出
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// 获取 Agent 脚本路径
    fn get_agent_script_path(&self) -> anyhow::Result<PathBuf> {
        let root = self.get_project_root()?;
        Ok(root.join("scripts").join("agent_server.py"))
    }

    /// 获取项目根目录
    fn get_project_root(&self) -> anyhow::Result<PathBuf> {
        // 尝试从环境变量获取，或使用当前目录
        if let Ok(dir) = std::env::current_dir() {
            if dir.join("scripts").join("agent_server.py").exists() {
                return Ok(dir);
            }
        }
        
        // 尝试从可执行文件位置推断
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                // 开发模式: target/debug/xhs-rs.exe -> 项目根目录
                let dev_root = parent.parent().and_then(|p| p.parent());
                if let Some(root) = dev_root {
                    if root.join("scripts").join("agent_server.py").exists() {
                        return Ok(root.to_path_buf());
                    }
                }
            }
        }
        
        // 默认使用当前目录
        Ok(std::env::current_dir()?)
    }
}

impl Default for AgentManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for AgentManager {
    fn drop(&mut self) {
        self.stop();
    }
}

/// 全局 Agent 管理器实例
static AGENT: once_cell::sync::Lazy<AgentManager> = 
    once_cell::sync::Lazy::new(AgentManager::new);

/// 启动 Agent（供外部调用）
pub fn start_agent() -> anyhow::Result<()> {
    AGENT.start()
}

/// 停止 Agent（供外部调用）
pub fn stop_agent() {
    AGENT.stop()
}

/// 检查 Agent 状态
pub fn is_agent_running() -> bool {
    AGENT.is_running()
}
