use crate::portfwd::PortForward;
use crate::ssh::ControlMasterStatus;

/// Application state
pub struct App {
    /// The SSH host being managed
    pub hostname: String,
    /// Path to the control socket
    pub control_path: String,
    /// Current control master status
    pub master_status: ControlMasterStatus,
    /// List of active port forwards
    pub forwards: Vec<PortForward>,
    /// Currently selected index in the forwards list
    pub selected: usize,
    /// Whether the app should quit
    pub should_quit: bool,
}

impl App {
    pub fn new(
        hostname: String,
        control_path: String,
        master_status: ControlMasterStatus,
        forwards: Vec<PortForward>,
    ) -> Self {
        Self {
            hostname,
            control_path,
            master_status,
            forwards,
            selected: 0,
            should_quit: false,
        }
    }

    /// Handle a key press event
    pub fn on_key(&mut self, key: char) {
        match key {
            'q' => self.should_quit = true,
            'j' => self.select_next(),
            'k' => self.select_prev(),
            _ => {}
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if !self.forwards.is_empty() {
            self.selected = (self.selected + 1) % self.forwards.len();
        }
    }

    /// Move selection up
    pub fn select_prev(&mut self) {
        if !self.forwards.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.forwards.len() - 1);
        }
    }
}
