use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, Window,
    COLOR_BLACK, COLOR_WHITE,
};
use std::{env, io::Error};

use node_script_list::{
    execute_command, get_package_manager_prefix, parse_package_json_file, sort_command_list,
    CommandPrefix, Mode,
};

fn main() {
    match App::new() {
        Ok(app) => {
            run_app(
                app.highlighted_command_index,
                app.commands,
                app.window,
                app.mode,
                app.filter_string,
            );
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };
}

fn run_app(
    mut highlighted_command_index: usize,
    commands: Vec<String>,
    window: pancurses::Window,
    mut mode: Mode,
    mut filter_string: String,
) {
    let mut quit = false;
    let mut filtered_commands: Vec<String> = commands.clone();
    while !quit {
        display_commands(highlighted_command_index, &filtered_commands, &window);
        display_filter_value(&filter_string, &window, filtered_commands.len() as i32);
        handle_keyboard_input(
            &mut highlighted_command_index,
            &commands,
            &window,
            &mut mode,
            &mut filter_string,
            &mut quit,
            &mut filtered_commands,
        );
    }
}

enum ColorScheme {
    Regular = 0,
    Highlighted = 1,
}

impl ColorScheme {
    fn init() {
        init_pair(Self::Regular as i16, COLOR_WHITE, COLOR_BLACK);
        init_pair(Self::Highlighted as i16, COLOR_BLACK, COLOR_WHITE);
    }

    fn pair(self) -> ColorPair {
        ColorPair(self as u8)
    }
}

struct App {
    highlighted_command_index: usize,
    commands: Vec<String>,
    window: Window,
    mode: Mode,
    filter_string: String,
}

impl App {
    fn new() -> Result<Self, String> {
        let current_directory =
            env::current_dir().map_err(|e| format!("Failed to get current directory {}", e))?;
        let json_value = parse_package_json_file(&current_directory)
            .ok_or_else(|| "No package.json found or failed to parse".to_string())?;
        let mut highlighted_command_index: usize = 0;
        let package_manager_prefix = get_package_manager_prefix(&current_directory);
        let prefixed_script_list: Vec<String> = json_value.prefix_command(package_manager_prefix);
        let commands = sort_command_list(prefixed_script_list);
        let window = initscr();
        let mut mode: Mode = Mode::DEFAULT;
        let mut filter_string: String = String::new();
        noecho();
        curs_set(0);
        start_color();
        ColorScheme::init();

        Ok(App {
            highlighted_command_index,
            commands,
            window,
            mode,
            filter_string,
        })
    }
}

fn handle_keyboard_input(
    highlighted_command_index: &mut usize,
    commands: &Vec<String>,
    window: &pancurses::Window,
    mode: &mut Mode,
    filter_string: &mut String,
    quit: &mut bool,
    filtered_commands: &mut Vec<String>,
) {
    let key = window.getch();
    match *mode {
        Mode::FILTER => handle_filter_mode(
            *highlighted_command_index,
            commands,
            window,
            mode,
            filter_string,
            quit,
            filtered_commands,
            key,
        ),
        Mode::DEFAULT => handle_default_mode(highlighted_command_index, commands, mode, quit, key),
    }
}

fn handle_default_mode(
    highlighted_command_index: &mut usize,
    command: &Vec<String>,
    mode: &mut Mode,
    quit: &mut bool,
    key: Option<Input>,
) {
    match key {
        Some(Input::Character('q')) => {
            *quit = true;
            endwin();
        }
        Some(Input::Character('w')) => {
            if *highlighted_command_index > 0 {
                *highlighted_command_index -= 1;
            } else {
                *highlighted_command_index = command.len() - 1;
            }
        }
        Some(Input::Character('s')) => {
            *highlighted_command_index += 1;
            if *highlighted_command_index > command.len() - 1 {
                *highlighted_command_index = 0;
            }
        }
        Some(Input::Character('\n')) => {
            *quit = true;
            endwin();
            execute_command(&*command[*highlighted_command_index]);
        }
        Some(Input::Character('f')) => {
            *mode = Mode::FILTER;
            *highlighted_command_index = 0;
        }
        _ => {}
    }
}

fn handle_filter_mode(
    selected_command_index: usize,
    sorted_script_list: &Vec<String>,
    window: &pancurses::Window,
    mode: &mut Mode,
    filter_string: &mut String,
    quit: &mut bool,
    filtered_commands: &mut Vec<String>,
    key: Option<Input>,
) {
    match key {
        None => {}
        Some(Input::Character('\x1B')) => {
            *mode = Mode::DEFAULT;
            filter_string.clear();
            *filtered_commands = sorted_script_list.clone();
            display_commands(selected_command_index, &*filtered_commands, window);
        }
        Some(Input::Character('\n')) => {
            *quit = true;
            endwin();
            execute_command(&*sorted_script_list[selected_command_index]);
        }
        Some(Input::KeyBackspace) => {
            filter_string.pop();
            let filter_pattern: &str = &*filter_string;
            *filtered_commands = sorted_script_list.clone();
            filter_commands(
                selected_command_index,
                window,
                filtered_commands,
                filter_pattern,
            );
        }
        Some(Input::Character('\x7f')) => {
            filter_string.pop();
            let filter_pattern: &str = &*filter_string;
            *filtered_commands = sorted_script_list.clone();
            filter_commands(
                selected_command_index,
                window,
                filtered_commands,
                filter_pattern,
            );
        }
        Some(Input::KeyDC) => {
            filter_string.pop();
            let filter_pattern: &str = &*filter_string;
            *filtered_commands = sorted_script_list.clone();
            filter_commands(
                selected_command_index,
                window,
                filtered_commands,
                filter_pattern,
            );
        }
        Some(Input::Character(character)) => {
            if character.is_alphanumeric() || character == ' ' {
                filter_string.push(character);
                let filter_pattern: &str = &*filter_string;
                *filtered_commands = sorted_script_list.clone();
                filter_commands(
                    selected_command_index,
                    window,
                    filtered_commands,
                    filter_pattern,
                );
            }
        }
        _ => {
            todo!()
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
        let color_pair = if index == selected_command_index {
            ColorScheme::Highlighted.pair()
        } else {
            ColorScheme::Regular.pair()
        };
        window.attron(color_pair);
        window.addstr(&script_name);
        window.attroff(color_pair);
    }
}

fn display_filter_value(filter_pattern: &str, window: &pancurses::Window, last_index: i32) {
    window.mv(last_index + 1, 0 as i32);
    window.addstr(format!("filtered value: {}", filter_pattern));
    window.refresh();
}
