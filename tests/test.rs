#[cfg(test)]
mod tests {
    use node_script_list::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::tempdir;

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
