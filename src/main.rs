use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

use pancurses::{
    curs_set, endwin, init_pair, initscr, noecho, start_color, ColorPair, Input, COLOR_BLACK,
    COLOR_WHITE,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    author: Option<String>,
    scripts: Option<HashMap<String, String>>,
    dependencies: Option<HashMap<String, String>>,
}

pub trait CommandPrefix {
    fn prefix_command(&self, prefix: &'static str) -> Vec<String>;
}

impl CommandPrefix for PackageJson {
    fn prefix_command(&self, prefix: &'static str) -> Vec<String> {
        self.scripts
            .as_ref()
            .expect("PackageJson Parse Error: Scripts Unavailable")
            .iter()
            .map(|(script_name, _)| format!("{} {}", prefix, script_name))
            .collect()
    }
}

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHTED_PAIR: i16 = 1;

fn main() {
    let current_directory = env::current_dir().expect("Failed to get current directory");
    let json_value = match parse_package_json_file(&current_directory) {
        Some(value) => value,
        None => return,
    };

    let mut selected_command_index = 0;

    let package_manager_prefix = get_package_manager_prefix(&current_directory); // Pass current_directory
                                                                                 // Build the list of all the script names
    let prefixed_script_list: Vec<String> = json_value.prefix_command(package_manager_prefix);

    let sorted_script_list = sort_command_list(prefixed_script_list);

    // Display Script Commands
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

// Modified to accept a base_path
fn parse_package_json_file(base_path: &std::path::Path) -> Option<PackageJson> {
    // let current_directory = env::current_dir().expect("Failed to get current directory"); // Removed
    // Build the package.json file path
    let file_path = base_path.join("package.json"); // Use base_path
                                                    // Verify if file exists
    if !file_path.is_file() {
        println!("File does not exist at {:?}", file_path); // Added path for better debugging
        return None;
    }
    // Open File
    let mut file = File::open(file_path).expect("Failed to open file");
    // Read
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");
    // Parse JSON
    let json_value: PackageJson = serde_json::from_str(&json_string).expect("Failed to parse json");
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
}

// Modified to accept a base_path
fn get_package_manager_prefix(base_path: &std::path::Path) -> &'static str {
    if is_yarn_used(base_path) {
        // Pass base_path
        return "yarn";
    }
    if is_pnpm_used(base_path) {
        // Pass base_path
        return "pnpm run";
    }
    return "npm run";
}

// Modified to accept a base_path
fn is_npm_used(base_path: &std::path::Path) -> bool {
    // let current_directory = env::current_dir().expect("Failed to get current directory"); // Removed
    base_path.join("package-lock.json").exists() // Use base_path
}

// Modified to accept a base_path
fn is_pnpm_used(base_path: &std::path::Path) -> bool {
    // let current_directory = env::current_dir().expect("Failed to get current directory"); // Removed
    base_path.join("pnpm-lock.yml").exists() // Use base_path
}

// Modified to accept a base_path
fn is_yarn_used(base_path: &std::path::Path) -> bool {
    // let current_directory = env::current_dir().expect("Failed to get current directory"); // Removed
    base_path.join("yarn.lock").exists() // Use base_path
}

fn sort_command_list(command_list: Vec<String>) -> Vec<String> {
    let mut sorted_script_list: Vec<String> = command_list.iter().cloned().collect();
    sorted_script_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    sorted_script_list
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::{tempdir, NamedTempFile}; // Make sure tempfile is in your Cargo.toml dependencies

    // Helper function to create a dummy package.json for tests
    fn create_dummy_package_json(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
        let file_path = dir.join("package.json");
        fs::write(&file_path, content).expect("Unable to write dummy package.json");
        file_path
    }

    // Helper function to create dummy lock files
    fn create_dummy_lock_file(dir: &std::path::Path, filename: &str) -> std::path::PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, "").expect(&format!("Unable to write dummy {}", filename));
        file_path
    }

    #[test]
    fn test_prefix_command_with_scripts() {
        let mut scripts = HashMap::new();
        scripts.insert("start".to_string(), "node index.js".to_string());
        scripts.insert("build".to_string(), "webpack".to_string());
        let package_json = PackageJson {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            description: None,
            author: None,
            scripts: Some(scripts),
            dependencies: None,
        };

        let prefixed_commands = package_json.prefix_command("npm run");
        assert_eq!(prefixed_commands.len(), 2);
        assert!(prefixed_commands.contains(&"npm run start".to_string()));
        assert!(prefixed_commands.contains(&"npm run build".to_string()));
    }

    #[test]
    #[should_panic(expected = "PackageJson Parse Error: Scripts Unavailable")]
    fn test_prefix_command_no_scripts() {
        let package_json = PackageJson {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            description: None,
            author: None,
            scripts: None,
            dependencies: None,
        };

        package_json.prefix_command("npm run");
    }

    #[test]
    fn test_prefix_command_empty_scripts() {
        let scripts = HashMap::new();
        let package_json = PackageJson {
            name: Some("test-app".to_string()),
            version: Some("1.0.0".to_string()),
            description: None,
            author: None,
            scripts: Some(scripts),
            dependencies: None,
        };

        let prefixed_commands = package_json.prefix_command("npm run");
        assert!(prefixed_commands.is_empty());
    }

    #[test]
    fn test_parse_package_json_file_success() {
        let dir = tempdir().expect("Failed to create temp dir");
        let content = r#"{
            "name": "test-app",
            "version": "1.0.0",
            "scripts": {
                "start": "node index.js"
            }
        }"#;
        create_dummy_package_json(dir.path(), content);

        // Pass the temporary directory path directly
        let package_json =
            parse_package_json_file(dir.path()).expect("Failed to parse package.json");

        assert_eq!(package_json.name, Some("test-app".to_string()));
        assert_eq!(package_json.version, Some("1.0.0".to_string()));
        assert!(package_json.scripts.is_some());
        assert_eq!(
            package_json.scripts.unwrap()["start"],
            "node index.js".to_string()
        );

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_parse_package_json_file_not_found() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Pass the temporary directory path directly
        let package_json = parse_package_json_file(dir.path());
        assert!(package_json.is_none());

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    #[should_panic(expected = "Failed to parse json")]
    fn test_parse_package_json_file_invalid_json() {
        let dir = tempdir().expect("Failed to create temp dir");
        let content = r#"{
            "name": "test-app",
            "version": "1.0.0",
            "scripts": {
                "start": "node index.js"
            "#; // Malformed JSON
        create_dummy_package_json(dir.path(), content);

        // Pass the temporary directory path directly
        let _ = parse_package_json_file(dir.path()); // This should panic

        // dir.close() is handled by the Drop trait of tempdir
    }

    #[test]
    fn test_is_npm_used() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Test when package-lock.json exists
        create_dummy_lock_file(dir.path(), "package-lock.json");
        assert!(is_npm_used(dir.path())); // Pass the temp dir path

        // Test when package-lock.json does not exist
        fs::remove_file(dir.path().join("package-lock.json")).expect("Failed to remove file");
        assert!(!is_npm_used(dir.path())); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_is_pnpm_used() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Test when pnpm-lock.yml exists
        create_dummy_lock_file(dir.path(), "pnpm-lock.yml");
        assert!(is_pnpm_used(dir.path())); // Pass the temp dir path

        // Test when pnpm-lock.yml does not exist
        fs::remove_file(dir.path().join("pnpm-lock.yml")).expect("Failed to remove file");
        assert!(!is_pnpm_used(dir.path())); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_is_yarn_used() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Test when yarn.lock exists
        create_dummy_lock_file(dir.path(), "yarn.lock");
        assert!(is_yarn_used(dir.path())); // Pass the temp dir path

        // Test when yarn.lock does not exist
        fs::remove_file(dir.path().join("yarn.lock")).expect("Failed to remove file");
        assert!(!is_yarn_used(dir.path())); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_get_package_manager_prefix_npm() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Only npm lock file
        create_dummy_lock_file(dir.path(), "package-lock.json");
        assert_eq!(get_package_manager_prefix(dir.path()), "npm run"); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_get_package_manager_prefix_yarn() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Yarn lock file takes precedence
        create_dummy_lock_file(dir.path(), "yarn.lock");
        create_dummy_lock_file(dir.path(), "package-lock.json"); // Should be ignored
        assert_eq!(get_package_manager_prefix(dir.path()), "yarn"); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_get_package_manager_prefix_pnpm() {
        let dir = tempdir().expect("Failed to create temp dir");

        // pnpm lock file takes precedence over npm
        create_dummy_lock_file(dir.path(), "pnpm-lock.yml");
        create_dummy_lock_file(dir.path(), "package-lock.json"); // Should be ignored
        assert_eq!(get_package_manager_prefix(dir.path()), "pnpm run"); // Pass the temp dir path

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_get_package_manager_prefix_priority() {
        let dir = tempdir().expect("Failed to create temp dir");

        // Yarn > pnpm > npm
        create_dummy_lock_file(dir.path(), "package-lock.json");
        create_dummy_lock_file(dir.path(), "pnpm-lock.yml");
        create_dummy_lock_file(dir.path(), "yarn.lock");
        assert_eq!(get_package_manager_prefix(dir.path()), "yarn");

        fs::remove_file(dir.path().join("yarn.lock")).expect("Failed to remove file");
        assert_eq!(get_package_manager_prefix(dir.path()), "pnpm run");

        fs::remove_file(dir.path().join("pnpm-lock.yml")).expect("Failed to remove file");
        assert_eq!(get_package_manager_prefix(dir.path()), "npm run");

        dir.close().expect("Failed to clean up temp dir");
    }

    #[test]
    fn test_sort_command_list_basic() {
        let commands = vec![
            "c command".to_string(),
            "a command".to_string(),
            "b command".to_string(),
        ];
        let sorted = sort_command_list(commands);
        assert_eq!(
            sorted,
            vec![
                "a command".to_string(),
                "b command".to_string(),
                "c command".to_string(),
            ]
        );
    }

    #[test]
    fn test_sort_command_list_case_insensitive() {
        let commands = vec![
            "Z command".to_string(),
            "a command".to_string(),
            "B command".to_string(),
            "c command".to_string(),
        ];
        let sorted = sort_command_list(commands);
        assert_eq!(
            sorted,
            vec![
                "a command".to_string(),
                "B command".to_string(),
                "c command".to_string(),
                "Z command".to_string(),
            ]
        );
    }

    #[test]
    fn test_sort_command_list_empty() {
        let commands: Vec<String> = vec![];
        let sorted = sort_command_list(commands);
        assert!(sorted.is_empty());
    }

    #[test]
    fn test_sort_command_list_single_item() {
        let commands = vec!["single command".to_string()];
        let sorted = sort_command_list(commands);
        assert_eq!(sorted, vec!["single command".to_string()]);
    }
}
