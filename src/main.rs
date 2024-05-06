use std::path::Path;

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

}