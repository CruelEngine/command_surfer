use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
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


fn main() {
    let _result = parse_package();
}



fn parse_package() {
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

    // Display List of executable scripts

    for key in json_value.scripts.keys() {
        println!("npm run {}", key);
    }


}