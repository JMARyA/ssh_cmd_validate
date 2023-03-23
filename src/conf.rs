use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
/// Structure of Config file
pub struct Config {
    /// Default Command to run when nothing is provided
    pub default_command: Option<String>,
    /// List of allowed commands
    pub allowed_commands: Vec<CommandOption>,
    /// log file
    pub log_file: Option<String>,
}

#[derive(Serialize, Deserialize)]
/// command config
pub struct CommandOption {
    /// executable path
    pub executable: String,
    /// force a list of arguments, any if empty
    pub force_arguments: Option<Vec<String>>,
}
