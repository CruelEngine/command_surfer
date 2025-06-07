use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, Window,
    COLOR_BLACK, COLOR_WHITE,
};
use std::env;

use command_surfer::{
    execute_command, get_package_manager_prefix, parse_package_json_file, sort_command_list,
    CommandPrefix, Mode,
};

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
    quit: bool,
}

impl App {
    fn new() -> Result<Self, String> {
        let current_directory =
            env::current_dir().map_err(|e| format!("Failed to get current directory {}", e))?;
        let json_value = parse_package_json_file(&current_directory)
            .ok_or_else(|| "No package.json found or failed to parse".to_string())?;
        let package_manager_prefix = get_package_manager_prefix(&current_directory);
        let prefixed_script_list: Vec<String> = json_value.prefix_command(package_manager_prefix);
        let commands = sort_command_list(prefixed_script_list);
        let window = initscr();
        noecho();
        curs_set(0);
        start_color();
        ColorScheme::init();

        Ok(App {
            highlighted_command_index: 0,
            commands,
            window,
            mode: Mode::DEFAULT,
            filter_string: String::new(),
            quit: false,
        })
    }

    fn run(&mut self) {
        let filtered_commands: Vec<String> = self.commands.clone();
        while !self.quit {
            self.display_commands(&filtered_commands);
            match self.mode {
                Mode::FILTER => self.display_filter_value(filtered_commands.len() as i32),
                _ => {}
            }
            self.handle_keyboard_input();
        }
    }

    fn handle_keyboard_input(&mut self) {
        let key = self.window.getch();
        match self.mode {
            Mode::FILTER => self.handle_filter_mode(key),
            Mode::DEFAULT => self.handle_default_mode(key),
        }
    }

    fn handle_default_mode(&mut self, key: Option<Input>) {
        match key {
            Some(Input::Character('q')) => {
                self.quit = true;
                endwin();
            }
            Some(Input::Character('w')) => {
                if self.highlighted_command_index > 0 {
                    self.highlighted_command_index -= 1;
                } else {
                    self.highlighted_command_index = self.commands.len() - 1;
                }
            }
            Some(Input::Character('s')) => {
                self.highlighted_command_index += 1;
                if self.highlighted_command_index > self.commands.len() - 1 {
                    self.highlighted_command_index = 0;
                }
            }
            Some(Input::Character('\n')) => {
                self.quit = true;
                endwin();
                execute_command(&self.commands[self.highlighted_command_index]);
            }
            Some(Input::Character('f')) => {
                self.mode = Mode::FILTER;
                self.highlighted_command_index = 0;
            }
            _ => {}
        }
    }

    fn handle_filter_mode(&mut self, key: Option<Input>) {
        match key {
            None => {}
            Some(Input::Character('\x1B')) => {
                self.mode = Mode::DEFAULT;
                self.filter_string.clear();
                self.filter_commands(&self.filter_string);
            }
            Some(Input::Character('\n')) => {
                self.quit = true;
                endwin();
                execute_command(&self.commands[self.highlighted_command_index]);
            }
            Some(Input::KeyBackspace) => {
                self.filter_string.pop();
                let filter_pattern: &str = &self.filter_string;
                self.filter_commands(filter_pattern);
            }
            Some(Input::Character('\x7f')) => {
                self.filter_string.pop();
                let filter_pattern: &str = &self.filter_string;
                self.filter_commands(filter_pattern);
            }
            Some(Input::KeyDC) => {
                self.filter_string.pop();
                let filter_pattern: &str = &self.filter_string;
                self.filter_commands(filter_pattern);
            }
            Some(Input::Character(character)) => {
                if character.is_alphanumeric() || character == ' ' {
                    self.filter_string.push(character);
                    let filter_pattern: &str = &self.filter_string;
                    self.filter_commands(filter_pattern);
                }
            }
            _ => {
                todo!()
            }
        }
    }

    fn filter_commands(&self, filter_pattern: &str) {
        let filtered_commands = self
            .commands
            .clone()
            .iter()
            .filter(|comand| comand.contains(filter_pattern))
            .cloned()
            .collect();
        self.display_commands(&filtered_commands);
    }

    fn display_commands(&self, commands: &Vec<String>) {
        self.window.erase();
        self.window.mv(0, 0);
        for (index, script_name) in commands.iter().enumerate() {
            self.window.mv(index as i32, 0 as i32);
            let color_pair = if index == self.highlighted_command_index {
                ColorScheme::Highlighted.pair()
            } else {
                ColorScheme::Regular.pair()
            };
            self.window.attron(color_pair);
            self.window.addstr(&script_name);
            self.window.attroff(color_pair);
        }
    }

    fn display_filter_value(&self, last_index: i32) {
        self.window.mv(last_index + 1, 0 as i32);
        self.window
            .addstr(format!("filtered value: {}", self.filter_string));
        self.window.refresh();
    }
}

fn main() {
    match App::new() {
        Ok(mut app) => {
            app.run();
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    };
}
