use crate::machine::machine::Machine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiMode {
    Easy,
    Hard,
}

pub struct AppState {
    pub mode: UiMode,

    pub machine: Machine,

    // Could be reduced
    pub halted: bool,
    pub running: bool,

    pub program_name: Option<String>,

    // Potentially make it a bitfield
    pub show_memory: bool,
    pub show_registers: bool,
    pub show_cpu: bool,
    pub show_console: bool,

    pub status_message: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: UiMode::Easy,

            machine: Machine::new(),

            running: false,
            halted: false,

            program_name: None,

            show_memory: false,
            show_registers: false,
            show_cpu: false,
            show_console: true,

            status_message: "Ready".to_string(),
        }
    }
}
