use crate::portfwd::PortForward;
use crate::ssh::{self, ControlMasterStatus};

/// Current input mode
#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    AddingForward,
}

/// Application state
pub struct App {
    /// The SSH host being managed
    pub hostname: String,
    /// Current control master status
    pub master_status: ControlMasterStatus,
    /// List of active port forwards
    pub forwards: Vec<PortForward>,
    /// Currently selected index in the forwards list
    pub selected: usize,
    /// Current input mode
    pub input_mode: InputMode,
    /// Input buffer for text entry
    pub input_buffer: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Whether the app should quit
    pub should_quit: bool,
}

impl App {
    pub fn new(
        hostname: String,
        master_status: ControlMasterStatus,
        forwards: Vec<PortForward>,
    ) -> Self {
        Self {
            hostname,
            master_status,
            forwards,
            selected: 0,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            status_message: None,
            should_quit: false,
        }
    }

    /// Handle a key press event in normal mode
    pub fn on_key(&mut self, key: char) {
        // Clear status message on any key press
        self.status_message = None;

        match key {
            'q' => self.should_quit = true,
            'j' => self.select_next(),
            'k' => self.select_prev(),
            'a' => self.start_add_forward(),
            'd' => self.delete_selected_forward(),
            _ => {}
        }
    }

    /// Start adding a new forward
    fn start_add_forward(&mut self) {
        // Start control master if not running
        if matches!(self.master_status, ControlMasterStatus::NotRunning) {
            self.status_message = Some("Starting control master...".to_string());
            match ssh::start_control_master(&self.hostname) {
                Ok(pid) => {
                    self.master_status = ControlMasterStatus::Running { pid };
                    self.status_message = Some(format!("Control master started (PID: {})", pid));
                }
                Err(e) => {
                    self.status_message = Some(format!("Failed to start control master: {}", e));
                    return;
                }
            }
        }
        self.input_mode = InputMode::AddingForward;
        self.input_buffer.clear();
    }

    /// Delete the currently selected forward
    fn delete_selected_forward(&mut self) {
        if self.forwards.is_empty() {
            return;
        }

        let forward = &self.forwards[self.selected];
        let port = forward.local_port;

        match ssh::cancel_forward(&self.hostname, port) {
            Ok(()) => {
                self.forwards.remove(self.selected);
                // Adjust selection if needed
                if self.selected >= self.forwards.len() && !self.forwards.is_empty() {
                    self.selected = self.forwards.len() - 1;
                }
                self.status_message = Some(format!("Deleted forward: localhost:{}", port));
            }
            Err(e) => {
                self.status_message = Some(format!("Error: {}", e));
            }
        }
    }

    /// Handle input in AddingForward mode
    pub fn on_input_key(&mut self, key: char) {
        // Only allow digits for port number
        if key.is_ascii_digit() {
            self.input_buffer.push(key);
        }
    }

    /// Handle backspace in input mode
    pub fn on_input_backspace(&mut self) {
        self.input_buffer.pop();
    }

    /// Cancel input mode
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Submit the current input
    pub fn submit_input(&mut self) {
        if self.input_mode == InputMode::AddingForward {
            if let Ok(port) = self.input_buffer.parse::<u16>() {
                match ssh::add_forward(&self.hostname, port) {
                    Ok(()) => {
                        // Add to local list
                        self.forwards.push(PortForward { local_port: port });
                        self.forwards.sort_by_key(|f| f.local_port);
                        self.status_message = Some(format!("Added forward: localhost:{}", port));
                    }
                    Err(e) => {
                        self.status_message = Some(format!("Error: {}", e));
                    }
                }
            } else {
                self.status_message = Some("Invalid port number".to_string());
            }
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
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
