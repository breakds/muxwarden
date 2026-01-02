mod app;
mod portfwd;
mod ssh;
mod ui;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "muxwarden")]
#[command(about = "A TUI for managing SSH multiplex connections and port forwarding")]
struct Cli {
    /// The SSH host to manage
    host: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Add a port forward (one-shot, no TUI)
    Forward {
        /// Local port to forward
        port: u16,
    },
    /// Cancel a port forward (one-shot, no TUI)
    Cancel {
        /// Local port to cancel
        port: u16,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Query SSH configuration
    let config = ssh::SshConfig::query(&cli.host)?;

    if !config.has_control_path() {
        anyhow::bail!(
            "No ControlPath configured for host '{}'. \
             Please configure ControlPath in your ~/.ssh/config",
            cli.host
        );
    }

    let control_path = config.control_path.unwrap();

    // Handle CLI subcommands (one-shot mode)
    if let Some(command) = cli.command {
        return handle_cli_command(&cli.host, command);
    }

    // TUI mode: gather state and launch
    let master_status = ssh::check_control_master(&cli.host)?;
    let forwards = match &master_status {
        ssh::ControlMasterStatus::Running { pid } => portfwd::list_forwards_by_pid(*pid)?,
        ssh::ControlMasterStatus::NotRunning => vec![],
    };

    let mut app = app::App::new(cli.host, control_path, master_status, forwards);
    ui::run(&mut app)?;

    Ok(())
}

fn handle_cli_command(host: &str, command: Command) -> Result<()> {
    match command {
        Command::Forward { port } => {
            ssh::add_forward(host, port)?;
            println!("Added forward: localhost:{}", port);
        }
        Command::Cancel { port } => {
            ssh::cancel_forward(host, port)?;
            println!("Cancelled forward: localhost:{}", port);
        }
    }
    Ok(())
}
