use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, COLOR_BLACK,
    COLOR_WHITE,
};
use std::env;

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHTED_PAIR: i16 = 1;

use node_script_list::{
    execute_command, get_package_manager_prefix, parse_package_json_file, sort_command_list,
    CommandPrefix, ToolMode,
};

fn main() {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    let json_value = match parse_package_json_file(&current_directory) {
        Some(value) => value,
        None => return,
    };

    let mut selected_command_index = 0;

    let package_manager_prefix = get_package_manager_prefix(&current_directory);

    let prefixed_script_list: Vec<String> = json_value.prefix_command(package_manager_prefix);

    let sorted_script_list = sort_command_list(prefixed_script_list);

    let window = initscr();
    let mut mode: ToolMode = ToolMode::DEFAULT;
    let mut filter_string: String = String::new();
    noecho();

    curs_set(0);
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHTED_PAIR, COLOR_BLACK, COLOR_WHITE);

    // Display List of executable scripts
    let mut quit = false;
    let mut filtered_commands: Vec<String> = sorted_script_list.clone();
    while !quit {
        display_commands(selected_command_index, &filtered_commands, &window);
        display_filter_value(&filter_string, &window, filtered_commands.len() as i32);
        let key = window.getch();
        match mode {
            ToolMode::FILTER => match key {
                None => {}
                Some(Input::Character('\x1B')) => {
                    mode = ToolMode::DEFAULT;
                    filter_string.clear();
                    filtered_commands = sorted_script_list.clone();
                    display_commands(selected_command_index, &filtered_commands, &window);
                }
                Some(Input::Character('\n')) => {
                    quit = true;
                    endwin();
                    execute_command(&sorted_script_list[selected_command_index]);
                }
                Some(Input::KeyBackspace) => {
                    filter_string.pop();
                    let filter_pattern: &str = &filter_string;
                    filtered_commands = sorted_script_list.clone();
                    filter_commands(
                        selected_command_index,
                        &window,
                        &mut filtered_commands,
                        filter_pattern,
                    );
                }
                Some(Input::Character('\x7f')) => {
                    filter_string.pop();
                    let filter_pattern: &str = &filter_string;
                    filtered_commands = sorted_script_list.clone();
                    filter_commands(
                        selected_command_index,
                        &window,
                        &mut filtered_commands,
                        filter_pattern,
                    );
                }
                Some(Input::KeyDC) => {
                    filter_string.pop();
                    let filter_pattern: &str = &filter_string;
                    filtered_commands = sorted_script_list.clone();
                    filter_commands(
                        selected_command_index,
                        &window,
                        &mut filtered_commands,
                        filter_pattern,
                    );
                }
                Some(Input::Character(character)) => {
                    if character.is_alphanumeric() || character == ' ' {
                        filter_string.push(character);
                        let filter_pattern: &str = &filter_string;
                        filtered_commands = sorted_script_list.clone();
                        filter_commands(
                            selected_command_index,
                            &window,
                            &mut filtered_commands,
                            filter_pattern,
                        );
                    }
                }
                _ => {
                    todo!()
                }
            },
            ToolMode::DEFAULT => match key {
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
                Some(Input::Character('f')) => {
                    mode = ToolMode::FILTER;
                    selected_command_index = 0;
                }
                _ => {}
            },
        }
    }
}

fn filter_commands(
    selected_command_index: usize,
    window: &pancurses::Window,
    filtered_commands: &mut Vec<String>,
    filter_pattern: &str,
) {
    *filtered_commands = filtered_commands
        .iter()
        .filter(|comand| comand.contains(filter_pattern))
        .cloned()
        .collect();
    display_commands(selected_command_index, &*filtered_commands, window);
}

fn display_commands(
    selected_command_index: usize,
    sorted_script_list: &Vec<String>,
    window: &pancurses::Window,
) {
    window.erase();
    window.mv(0, 0);
    for (index, script_name) in sorted_script_list.iter().enumerate() {
        window.mv(index as i32, 0 as i32);
        let attribute = if index == selected_command_index {
            window.attron(ColorPair(HIGHLIGHTED_PAIR as u8));
        } else {
            window.attron(ColorPair(REGULAR_PAIR as u8));
        };
        window.addstr(&script_name);
        if index == selected_command_index {
            window.attroff(ColorPair(HIGHLIGHTED_PAIR as u8));
        } else {
            window.attroff(ColorPair(REGULAR_PAIR as u8));
        };
    }
}

fn display_filter_value(filter_pattern: &str, window: &pancurses::Window, last_index: i32) {
    window.mv(last_index + 1, 0 as i32);
    window.addstr(format!("filtered value: {}", filter_pattern));
    window.refresh();
}
