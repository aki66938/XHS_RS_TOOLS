//! Own the optional local Python agent; reuse an already running project agent.
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct AgentManager {
    process: Mutex<Option<Child>>,
}

fn local_agent_ready() -> bool {
    let address: SocketAddr = "127.0.0.1:8765".parse().unwrap();
    let Ok(mut socket) = TcpStream::connect_timeout(&address, Duration::from_millis(200)) else {
        return false;
    };
    let _ = socket.set_read_timeout(Some(Duration::from_millis(300)));
    let _ = socket.set_write_timeout(Some(Duration::from_millis(300)));
    if socket.write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").is_err() {
        return false;
    }
    let mut response = String::new();
    socket.read_to_string(&mut response).is_ok()
        && response.starts_with("HTTP/1.1 200")
        && response.contains("\"service\":\"xhs-browser-agent\"")
        && response.contains("\"backend\":\"nodriver\"")
}

impl AgentManager {
    pub fn new() -> Self {
        Self { process: Mutex::new(None) }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        if crate::config::is_container_mode() {
            info!("[AgentManager] Using configured external agent");
            return Ok(());
        }
        if local_agent_ready() {
            info!("[AgentManager] Reusing local nodriver agent; this process does not own it");
            return Ok(());
        }
        let root = self.get_project_root()?;
        let python = std::env::var("XHS_PYTHON").unwrap_or_else(|_| {
            let local = root.join(if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" });
            if local.exists() { local.to_string_lossy().into_owned() } else { "python3".to_string() }
        });
        let mut child = Command::new(python)
            .args(["-m", "uvicorn", "scripts.agent_server:app", "--host", "127.0.0.1", "--port", "8765"])
            .current_dir(root)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while Instant::now() < deadline {
            if let Some(status) = child.try_wait()? {
                anyhow::bail!("Python agent exited during startup: {}", status);
            }
            if local_agent_ready() {
                info!("[AgentManager] Agent is ready (PID: {})", child.id());
                *self.process.lock().unwrap() = Some(child);
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        let _ = child.kill();
        let _ = child.wait();
        anyhow::bail!("Python agent did not become ready within 15 seconds")
    }

    pub fn stop(&self) {
        let Some(mut child) = self.process.lock().unwrap().take() else { return };
        if matches!(child.try_wait(), Ok(Some(_))) { return; }
        // Let FastAPI's lifespan close Chrome and flush local drafts first.
        #[cfg(unix)]
        {
            let _ = Command::new("kill").args(["-TERM", &child.id().to_string()]).status();
            let deadline = Instant::now() + Duration::from_secs(10);
            while Instant::now() < deadline {
                if matches!(child.try_wait(), Ok(Some(_))) { return; }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill").args(["/PID", &child.id().to_string(), "/T"]).status();
        }
        if let Err(error) = child.kill() {
            warn!("[AgentManager] Agent termination: {}", error);
        }
        let _ = child.wait();
    }

    pub fn is_running(&self) -> bool {
        self.process.lock().unwrap().as_mut()
            .map(|child| matches!(child.try_wait(), Ok(None))).unwrap_or(false)
    }

    fn get_project_root(&self) -> anyhow::Result<PathBuf> {
        let current = std::env::current_dir()?;
        if current.join("scripts/agent_server.py").exists() { return Ok(current); }
        if let Ok(executable) = std::env::current_exe() {
            for ancestor in executable.ancestors().skip(1).take(4) {
                if ancestor.join("scripts/agent_server.py").exists() { return Ok(ancestor.to_path_buf()); }
            }
        }
        anyhow::bail!("Run from the XHS_RS_TOOLS project directory")
    }
}

impl Default for AgentManager {
    fn default() -> Self { Self::new() }
}
impl Drop for AgentManager {
    fn drop(&mut self) { self.stop(); }
}
static AGENT: once_cell::sync::Lazy<AgentManager> = once_cell::sync::Lazy::new(AgentManager::new);
pub fn start_agent() -> anyhow::Result<()> { AGENT.start() }
pub fn stop_agent() { AGENT.stop(); }
pub fn is_agent_running() -> bool { AGENT.is_running() }
