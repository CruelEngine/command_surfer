use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub scripts: Option<HashMap<String, String>>,
    pub dependencies: Option<HashMap<String, String>>,
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

pub fn parse_package_json_file(base_path: &std::path::Path) -> Option<PackageJson> {
    let file_path = base_path.join("package.json");
    if !file_path.is_file() {
        println!("File does not exist at {:?}", file_path);
        return None;
    }
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut json_string = String::new();
    file.read_to_string(&mut json_string)
        .expect("Failed to read file");
    let json_value: PackageJson = serde_json::from_str(&json_string).expect("Failed to parse json");
    Some(json_value)
}

pub fn execute_command(npm_command: &str) {
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

pub fn get_package_manager_prefix(base_path: &std::path::Path) -> &'static str {
    if is_yarn_used(base_path) {
        return "yarn";
    }
    if is_pnpm_used(base_path) {
        return "pnpm run";
    }
    return "npm run";
}

pub fn is_npm_used(base_path: &std::path::Path) -> bool {
    base_path.join("package-lock.json").exists()
}

pub fn is_pnpm_used(base_path: &std::path::Path) -> bool {
    base_path.join("pnpm-lock.yml").exists()
}

pub fn is_yarn_used(base_path: &std::path::Path) -> bool {
    base_path.join("yarn.lock").exists()
}

pub fn sort_command_list(command_list: Vec<String>) -> Vec<String> {
    let mut sorted_script_list: Vec<String> = command_list.iter().cloned().collect();
    sorted_script_list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    sorted_script_list
}
