//! MCP Server Process Management
//!
//! This module manages the lifecycle of the mimir-mcp child process,
//! allowing the Tauri app to start, stop, and monitor the MCP server.

use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Manages the MCP server child process
pub struct McpServerManager {
    /// The child process handle (if running)
    process: Mutex<Option<Child>>,
    /// Path to the database for the MCP server
    database_path: String,
}

impl McpServerManager {
    /// Create a new MCP server manager
    pub fn new(database_path: String) -> Arc<Self> {
        Arc::new(Self {
            process: Mutex::new(None),
            database_path,
        })
    }

    /// Check if the MCP server is currently running
    pub async fn is_running(&self) -> bool {
        let mut process = self.process.lock().await;
        if let Some(child) = process.as_mut() {
            // Try to check if process is still running
            match child.try_wait() {
                Ok(None) => true,   // Still running
                Ok(Some(_)) => {
                    // Process exited
                    *process = None;
                    false
                }
                Err(e) => {
                    warn!("Error checking MCP server status: {}", e);
                    false
                }
            }
        } else {
            false
        }
    }

    /// Start the MCP server process
    pub async fn start(&self) -> Result<(), String> {
        let mut process = self.process.lock().await;

        // Check if already running
        if let Some(child) = process.as_mut() {
            match child.try_wait() {
                Ok(None) => {
                    info!("MCP server is already running");
                    return Ok(());
                }
                Ok(Some(status)) => {
                    info!("Previous MCP server process exited with status: {:?}", status);
                }
                Err(e) => {
                    warn!("Error checking MCP server status: {}", e);
                }
            }
        }

        // Find the mimir-mcp binary
        let binary_path = Self::find_binary()?;

        info!(
            "Starting MCP server: {} --database {}",
            binary_path, self.database_path
        );

        // Spawn the MCP server process
        let child = Command::new(&binary_path)
            .arg("--database")
            .arg(&self.database_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Let stderr go to our logs
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("Failed to start MCP server: {}", e))?;

        let pid = child.id();
        *process = Some(child);

        info!("MCP server started with PID: {:?}", pid);
        Ok(())
    }

    /// Stop the MCP server process
    pub async fn stop(&self) -> Result<(), String> {
        let mut process = self.process.lock().await;

        if let Some(mut child) = process.take() {
            info!("Stopping MCP server...");

            // Try graceful shutdown first
            if let Err(e) = child.kill().await {
                warn!("Error killing MCP server process: {}", e);
            }

            // Wait for it to exit
            match child.wait().await {
                Ok(status) => {
                    info!("MCP server stopped with status: {:?}", status);
                }
                Err(e) => {
                    warn!("Error waiting for MCP server to stop: {}", e);
                }
            }
        } else {
            debug!("MCP server was not running");
        }

        Ok(())
    }

    /// Restart the MCP server
    pub async fn restart(&self) -> Result<(), String> {
        self.stop().await?;
        self.start().await
    }

    /// Find the mimir-mcp binary path
    fn find_binary() -> Result<String, String> {
        // In development, look for the binary in target/debug or target/release
        if cfg!(debug_assertions) {
            // Development mode - look in target/debug
            let debug_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .map(|p| p.join("mimir-mcp"));

            if let Some(path) = debug_path {
                if path.exists() {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }

        // Try to find alongside the current executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let sibling_path = exe_dir.join("mimir-mcp");
                if sibling_path.exists() {
                    return Ok(sibling_path.to_string_lossy().to_string());
                }

                // On macOS, the binary might be in the Resources folder
                #[cfg(target_os = "macos")]
                {
                    // Check if we're in a .app bundle
                    if let Some(resources_dir) = exe_dir.parent().map(|p| p.join("Resources")) {
                        let resources_path = resources_dir.join("mimir-mcp");
                        if resources_path.exists() {
                            return Ok(resources_path.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }

        // Fall back to PATH lookup
        if let Ok(path) = which::which("mimir-mcp") {
            return Ok(path.to_string_lossy().to_string());
        }

        // Last resort: just use the name and hope it's in PATH
        error!("Could not find mimir-mcp binary, will try PATH lookup");
        Ok("mimir-mcp".to_string())
    }
}

impl Drop for McpServerManager {
    fn drop(&mut self) {
        // Note: kill_on_drop(true) handles cleanup, but we log it
        debug!("McpServerManager dropped, child process will be cleaned up");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_is_running_when_not_started() {
        let manager = McpServerManager::new("/tmp/test.db".to_string());
        assert!(!manager.is_running().await);
    }

    #[tokio::test]
    async fn test_stop_when_not_running() {
        let manager = McpServerManager::new("/tmp/test.db".to_string());
        // Should not error when stopping a non-running server
        assert!(manager.stop().await.is_ok());
    }
}
