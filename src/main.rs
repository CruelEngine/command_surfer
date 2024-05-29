use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ncurses::addstr;
use ncurses::mv;
use ncurses::{
    attroff, attron, curs_set, endwin, erase, getch, init_pair, initscr, noecho, refresh,
    start_color, COLOR_BLACK, COLOR_PAIR, COLOR_WHITE, CURSOR_VISIBILITY,
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
    parse_package();
}

fn parse_package() {
    let mut quit = false;

    initscr();
    noecho();

    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHTED_PAIR, COLOR_BLACK, COLOR_WHITE);
    let current_directory = env::current_dir().expect("Failed to get current directory");
    // Build the package.json file path
    let file_path = current_directory.join("package.json");

    // Verify if file exists
    if !file_path.is_file() {
        println!("File does not exist");
        return;
    }


    get_package_manager_prefix();
    // Open File
    let mut file = File::open(file_path).expect("Failed to open file");

    // Read

    let mut json_string = String::new();

    file.read_to_string(&mut json_string)
        .expect("Failed to read file");

    // Parse JSON

    let json_value: Package = serde_json::from_str(&json_string).expect("Failed to parse json");

    let mut selected_command_index = 0;

    let package_manager_prefix = get_package_manager_prefix();
    // Build the list of all the script names
    let script_list: Vec<String> = json_value
        .scripts
        .iter()
        .map(|(key, _)| format!("{} {}", package_manager_prefix,key))
        .collect();
    // Display List of executable scripts
    while !quit {
        erase();
        mv(0, 0);
        for (index, key) in script_list.iter().enumerate() {
            mv(index as i32, 0 as i32);
            let attribute = if index == selected_command_index {
                attron(COLOR_PAIR(HIGHLIGHTED_PAIR));
            } else {
                attron(COLOR_PAIR(REGULAR_PAIR));
            };
            addstr(&key).unwrap();
            if index == selected_command_index {
                attroff(COLOR_PAIR(HIGHLIGHTED_PAIR));
            } else {
                attroff(COLOR_PAIR(REGULAR_PAIR));
            };
        }
        refresh();
        let key = getch();
        match key as u8 as char {
            'q' => {
                quit = true;
                endwin();
            }
            'w' => {
                if selected_command_index > 0 {
                    selected_command_index -= 1;
                } else {
                    selected_command_index = script_list.len() - 1;
                }
            }
            's' => {
                selected_command_index += 1;
                if selected_command_index > script_list.len() - 1 {
                    selected_command_index = 0;
                }
            }
            '\n' => {
                quit = true;
                endwin();
                execute_command(&script_list[selected_command_index]);
            }
            _ => {}
        }
    }
}

fn execute_command(npm_command: &str) {
    let mut command = Command::new("sh");
    command.arg("-c").arg(npm_command);
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command
        .spawn()
        .expect("failed to spawn sh process")
        .wait()
        .expect("failed to wait for sh process");
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


fn is_pnpm_used() ->bool {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    return current_directory.join("pnpm-lock.yml").exists();
}


fn is_yarn_used() -> bool {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    return current_directory.join("yarn.lock").exists();
}