use anyhow::{Context, Result};
use std::process::Command;

/// Result of checking for an active control master
#[derive(Debug)]
pub enum ControlMasterStatus {
    /// Control master is running with the given PID
    Running { pid: u32 },
    /// No control master is active
    NotRunning,
}

/// Information about SSH configuration for a host
#[derive(Debug)]
pub struct SshConfig {
    pub control_path: Option<String>,
}

impl SshConfig {
    /// Query SSH configuration for a host using `ssh -G`
    pub fn query(hostname: &str) -> Result<Self> {
        let output = Command::new("ssh")
            .args(["-G", hostname])
            .output()
            .context("Failed to execute ssh -G")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("ssh -G failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let control_path = parse_control_path(&stdout);

        Ok(SshConfig { control_path })
    }

    /// Check if this host has a control path configured
    pub fn has_control_path(&self) -> bool {
        self.control_path
            .as_ref()
            .is_some_and(|p| p != "none" && !p.is_empty())
    }
}

/// Parse the controlpath from ssh -G output
fn parse_control_path(output: &str) -> Option<String> {
    for line in output.lines() {
        if let Some(path) = line.strip_prefix("controlpath ") {
            let path = path.trim();
            if path != "none" && !path.is_empty() {
                return Some(path.to_string());
            }
        }
    }
    None
}

/// Check if a control master is running for the given host
pub fn check_control_master(hostname: &str) -> Result<ControlMasterStatus> {
    let output = Command::new("ssh")
        .args(["-O", "check", hostname])
        .output()
        .context("Failed to execute ssh -O check")?;

    // ssh -O check outputs to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        // Parse PID from "Master running (pid=12345)"
        if let Some(pid) = parse_master_pid(&stderr) {
            return Ok(ControlMasterStatus::Running { pid });
        }
        // Success but couldn't parse PID - shouldn't happen, but handle it
        anyhow::bail!("Control master running but couldn't parse PID from: {}", stderr);
    }

    // Non-zero exit means no control master
    Ok(ControlMasterStatus::NotRunning)
}

/// Parse PID from "Master running (pid=12345)" message
fn parse_master_pid(output: &str) -> Option<u32> {
    // Look for "pid=" followed by digits
    let pid_marker = "pid=";
    if let Some(start) = output.find(pid_marker) {
        let after_marker = &output[start + pid_marker.len()..];
        // Extract digits until we hit a non-digit
        let pid_str: String = after_marker.chars().take_while(|c| c.is_ascii_digit()).collect();
        return pid_str.parse().ok();
    }
    None
}

/// Start a control master for the given host
/// Returns the PID of the new master
pub fn start_control_master(hostname: &str) -> Result<u32> {
    // Start ssh in master mode, backgrounded, no command
    let output = Command::new("ssh")
        .args(["-fNM", hostname])
        .output()
        .context("Failed to execute ssh -fNM")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start control master: {}", stderr.trim());
    }

    // Give it a moment to establish, then check for the PID
    std::thread::sleep(std::time::Duration::from_millis(500));

    match check_control_master(hostname)? {
        ControlMasterStatus::Running { pid } => Ok(pid),
        ControlMasterStatus::NotRunning => {
            anyhow::bail!("Control master started but not detected")
        }
    }
}

/// Add a port forward via the control master
/// Maps local_port -> localhost:local_port on remote
pub fn add_forward(hostname: &str, port: u16) -> Result<()> {
    let forward_spec = format!("{}:localhost:{}", port, port);

    let output = Command::new("ssh")
        .args(["-O", "forward", "-L", &forward_spec, hostname])
        .output()
        .context("Failed to execute ssh -O forward")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to add forward: {}", stderr.trim());
    }

    Ok(())
}

/// Cancel a port forward via the control master
pub fn cancel_forward(hostname: &str, port: u16) -> Result<()> {
    let forward_spec = format!("{}:localhost:{}", port, port);

    let output = Command::new("ssh")
        .args(["-O", "cancel", "-L", &forward_spec, hostname])
        .output()
        .context("Failed to execute ssh -O cancel")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to cancel forward: {}", stderr.trim());
    }

    Ok(())
}
