use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, COLOR_BLACK,
    COLOR_PAIR, COLOR_WHITE,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    description: Option<String>,
    author: Option<String>,
    scripts: HashMap<String, String>,
    dependencies: HashMap<String, String>,
}

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHTED_PAIR: i16 = 1;

fn main() {
    let json_value = match parse_package_json_file() {
        Some(value) => value,
        None => return,
    };

    let mut selected_command_index = 0;

    let package_manager_prefix = get_package_manager_prefix();
    // Build the list of all the script names
    let script_list: Vec<String> = json_value
        .scripts
        .iter()
        .map(|(key, _)| format!("{} {}", package_manager_prefix, key))
        .collect();

    let mut sorted_script_list: Vec<String> = script_list.iter().cloned().collect();
    sorted_script_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

    let window = initscr();
    noecho();

    curs_set(0);
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHTED_PAIR, COLOR_BLACK, COLOR_WHITE);

    // Display List of executable scripts
    let mut quit = false;
    while !quit {
        window.erase();
        window.mv(0, 0);
        for (index, key) in sorted_script_list.iter().enumerate() {
            window.mv(index as i32, 0 as i32);
            let attribute = if index == selected_command_index {
                window.attron(ColorPair(HIGHLIGHTED_PAIR as u8));
            } else {
                window.attron(ColorPair(REGULAR_PAIR as u8));
            };
            window.addstr(&key);
            if index == selected_command_index {
                window.attroff(ColorPair(HIGHLIGHTED_PAIR as u8));
            } else {
                window.attroff(ColorPair(REGULAR_PAIR as u8));
            };
        }
        window.refresh();
        let key = window.getch();
        match key {
            Some(Input::Character('q')) => {
                quit = true;
                endwin();
            }
            Some(Input::Character('w')) => {
                if selected_command_index > 0 {
                    selected_command_index -= 1;
                } else {
                    selected_command_index = sorted_script_list.len() - 1;
                }
            }
            Some(Input::Character('s')) => {
                selected_command_index += 1;
                if selected_command_index > sorted_script_list.len() - 1 {
                    selected_command_index = 0;
                }
            }
            Some(Input::Character('\n')) => {
                quit = true;
                endwin();
                execute_command(&sorted_script_list[selected_command_index]);
            }
            _ => {}
        }
    }
}

fn parse_package_json_file() -> Option<Package> {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    // Build the package.json file path
    let file_path = current_directory.join("package.json");
    // Verify if file exists
    if !file_path.is_file() {
        println!("File does not exist");
        return None;
    }
    // Open File
    let mut file = File::open(file_path).expect("Failed to open file");
    // Read
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");
    // Parse JSON
    let json_value: Package = serde_json::from_str(&json_string).expect("Failed to parse json");
    Some(json_value)
}

fn execute_command(npm_command: &str) {
    let mut command: Command;
    if cfg!(target_os = "windows") {
        command = Command::new("cmd");
        command.arg("/C").arg(npm_command);
    } else {
        command = Command::new("sh");
        command.arg("-c").arg(npm_command);
    }
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
        .spawn()
        .expect("failed to spawn sh process")
        .wait()
        .expect("failed to wait for sh process");

    append_command_to_history(npm_command);
}

fn append_command_to_history(npm_command: &str) {
    let shell = env::var("SHELL").unwrap_or_else(|_| String::from("/bin/bash"));
    let history_file_path = if shell.contains("zsh") {
        format!("{}/.zsh_history", env::var("HOME").unwrap())
    } else {
        format!("{}/.bash_history", env::var("HOME").unwrap())
    };
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(history_file_path)
        .unwrap();

    if shell.contains("zsh") {
        // Zsh needs timestamp format
        writeln!(
            file,
            ": {}:0;{}",
            chrono::Utc::now().timestamp(),
            npm_command
        )
        .unwrap();
    } else {
        // Bash is simple :)
        writeln!(file, "{}", npm_command).unwrap();
    }
}

fn get_package_manager_prefix() -> &'static str {
    if is_yarn_used() {
        return "yarn";
    }
    if is_pnpm_used() {
        return "pnpm run";
    }
    return "npm run";
}

fn is_npm_used() -> bool {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    return current_directory.join("package-lock.json").exists();
}

fn is_pnpm_used() -> bool {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    return current_directory.join("pnpm-lock.yml").exists();
}

fn is_yarn_used() -> bool {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    return current_directory.join("yarn.lock").exists();
}
