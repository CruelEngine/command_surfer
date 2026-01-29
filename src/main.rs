use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, Window,
    COLOR_BLACK, COLOR_WHITE,
};
use std::env;

use clap::Parser;
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

#[derive(Parser)]
struct Cli {
    /// The path to the file
    #[arg(short = 'f', long)]
    filter: Option<String>,
}

impl App {
    fn new() -> Result<Self, String> {
        let args = Cli::parse();
        let current_directory =
            env::current_dir().map_err(|e| format!("Failed to get current directory {}", e))?;
        let json_value = parse_package_json_file(&current_directory)
            .ok_or_else(|| "No package.json found or failed to parse".to_string())?;
        let package_manager_prefix = get_package_manager_prefix(&current_directory);
        let prefixed_script_list: Vec<String> = json_value
            .prefix_command(package_manager_prefix)
            .into_iter()
            .filter(|command| match args.filter.as_ref() {
                Some(term) => command.contains(term),
                None => true,
            })
            .collect();
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
        while !self.quit {
            let filtered_commands = self
                .commands
                .clone()
                .iter()
                .filter(|comand| comand.contains(&self.filter_string))
                .cloned()
                .collect();
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
        self.handle_default_mode(key);
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

    fn display_commands(&self, commands: &Vec<String>) {
        self.window.erase();
        let (max_y, _) = self.window.get_max_yx();
        let display_height = (max_y - 1).max(1) as usize;

        let scroll_offset = if self.highlighted_command_index >= display_height {
            self.highlighted_command_index - display_height + 1
        } else {
            0
        };

        let visible_commands = commands
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(display_height);

        for (index, script_name) in visible_commands {
            let relative_y = (index - scroll_offset) as i32;
            self.window.mv(relative_y, 0);

            let color_pair = if index == self.highlighted_command_index {
                ColorScheme::Highlighted.pair()
            } else {
                ColorScheme::Regular.pair()
            };

            self.window.attron(color_pair);
            self.window.addstr(script_name);
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
