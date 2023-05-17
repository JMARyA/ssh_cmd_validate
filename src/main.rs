mod conf;
use chrono::Local;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};

fn setup_args() -> conf::Config {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!("Usage: {} [CONFIG_FILE]", env!("CARGO_PKG_NAME"));
        std::process::exit(1);
    }

    let config_file = args.first().unwrap();

    let config: Result<conf::Config, _> =
        if let Ok(file_content) = std::fs::read_to_string(config_file) {
            serde_json::from_str(&file_content)
        } else {
            eprintln!("Config file could not be opened");
            std::process::exit(1);
        };

    if config.is_err() {
        eprintln!("config file not valid");
        std::process::exit(1);
    }

    config.unwrap()
}

pub fn run_command(command: &String) {
    let mut cmd = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("could not spawn command");

    cmd.wait().expect("command failed");
}

fn main() {
    let config = setup_args();

    let mut log_file = if config.log_file.is_some() {
        Some(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(config.log_file.clone().unwrap())
                .expect("log file could not be opened"),
        )
    } else {
        None
    };
    let log_enabled = config.log_file.is_some();

    // get ssh command
    let ssh_original_command = {
        if let Ok(ssh_cmd) = std::env::var("SSH_ORIGINAL_COMMAND") {
            ssh_cmd
        } else {
            // run default if nothing was provided
            if let Some(default_cmd) = config.default_command {
                run_command(&default_cmd);

                if log_enabled {
                    writeln!(
                        &mut log_file.as_mut().unwrap(),
                        "{} - User \"{}\" [{}] Executed command: {}",
                        Local::now(),
                        std::env::var("USER").unwrap_or(String::new()),
                        std::env::var("SSH_CLIENT")
                            .unwrap()
                            .split(' ')
                            .next()
                            .unwrap(),
                        default_cmd
                    )
                    .expect("error writing to log file");
                }
            }
            std::process::exit(0);
        }
    };

    if log_enabled {
        writeln!(
            &mut log_file.as_mut().unwrap(),
            "{} - User \"{}\" [{}] Attempted command: {}",
            Local::now(),
            std::env::var("USER").unwrap_or(String::new()),
            std::env::var("SSH_CLIENT")
                .unwrap()
                .split(' ')
                .next()
                .unwrap(),
            ssh_original_command
        )
        .expect("error writing to log file");
    }

    let (exec, args) = {
        let mut split = ssh_original_command.split_whitespace();
        (split.next().unwrap(), split.collect::<Vec<&str>>())
    };

    let which_output = Command::new("which")
        .arg(exec)
        .output()
        .expect("could not get command path");

    let cmd_full_path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();

    if let Some(command_option) = config
        .allowed_commands
        .iter()
        .find(|x| x.executable == cmd_full_path)
    {
        let exec_cmd = match &command_option.force_arguments {
            Some(forced_args) => format!("{} {}", cmd_full_path, forced_args.join(" ")),
            None => format!("{} {}", cmd_full_path, args.join(" ")),
        };

        run_command(&exec_cmd);

        if log_enabled {
            writeln!(
                &mut log_file.as_mut().unwrap(),
                "{} - User \"{}\" [{}] Executed command: {}",
                Local::now(),
                std::env::var("USER").unwrap_or(String::new()),
                std::env::var("SSH_CLIENT")
                    .unwrap()
                    .split(' ')
                    .next()
                    .unwrap(),
                exec_cmd
            )
            .expect("error writing to log file");
        }
    } else {
        if log_enabled {
            writeln!(
                &mut log_file.unwrap(),
                "{} - User \"{}\" [{}] Denied attempt: {}",
                Local::now(),
                std::env::var("USER").unwrap_or(String::new()),
                std::env::var("SSH_CLIENT")
                    .unwrap()
                    .split(' ')
                    .next()
                    .unwrap(),
                ssh_original_command
            )
            .expect("error writing to log file");
        }
        println!("Access denied");
    }
}
