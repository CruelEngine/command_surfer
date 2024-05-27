use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use ncurses::{COLOR_BLACK, COLOR_WHITE, curs_set, CURSOR_VISIBILITY, endwin, init_pair, initscr, noecho, refresh, start_color, getch, attron, COLOR_PAIR};
use ncurses::addstr;
use ncurses::mv;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
    description: String,
    author: String,
    scripts: HashMap<String, String>,
    dependencies: HashMap<String, String>
}

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHTED_PAIR: i16 = 1;


fn main() {
    let _result = parse_package();
}



fn parse_package() {
    let mut quit = false;

    initscr();
    noecho();

    addstr("Hello World").unwrap();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHTED_PAIR, COLOR_BLACK, COLOR_WHITE);
    // Build the package.json file path
    let file_path = Path::new("package.json");

    // Verify if file exists
    if !file_path.is_file() {
        println!("File does not exist");
        return;
    }


    // Open File
    let mut file = File::open(file_path).expect("Failed to open file");

    // Read

    let mut json_string = String::new();

    file.read_to_string(&mut json_string).expect("Failed to read file");


    // Parse JSON

    let json_value: Package = serde_json::from_str(&json_string).expect("Failed to parse json");

    let mut selected_command_index = 0;

    // Display List of executable scripts

    let script_list = json_value.scripts.keys();
    for (_index, key) in json_value.scripts.keys().enumerate() {
        let script_name: String = format!("npm run {}", key);
        mv(_index as i32, 0 as i32);
        attron(COLOR_PAIR(REGULAR_PAIR));
        addstr(&script_name);
        println!("{}", script_name);
    }
    while !quit {

    let key = getch();

        match key as u8 as char {
            'q' => quit = true,
            'w' => {
                selected_command_index -= 1;
                if selected_command_index < 0{
                    selected_command_index = script_list.len() - 1;
                }
            },
            's' => {
                selected_command_index += 1;
                if script_list.len() < selected_command_index {
                    selected_command_index = 0;
                }
            },
            '\n' => {
                println!("{}",selected_command_index);
            },
            _ => {

            }
        }
    }
    endwin();

}

fn execute_command(npm_command: &str) {
    let mut command = Command::new("sh");
    command.arg("-c").arg(npm_command);
    command.stdin(Stdio::inherit()).stdout(Stdio::inherit()).stderr(Stdio::inherit());
    command.spawn().expect("failed to spawn sh process").wait().expect("failed to wait for sh process");
}