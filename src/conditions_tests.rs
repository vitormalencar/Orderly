#[cfg(test)]
mod tests {
    use crate::conditions::*;
    use crate::config;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::{tempdir, TempDir};

    // Helper function to create a temporary file with content
    fn create_temp_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path).unwrap();
        write!(file, "{}", content).unwrap();
        file_path
    }

    // Helper function to create a temporary directory
    fn create_temp_dir(parent: &Path, name: &str) -> PathBuf {
        let dir_path = parent.join(name);
        fs::create_dir_all(&dir_path).unwrap();
        dir_path
    }

    // Helper function to setup test environment
    fn setup_test_env() -> TempDir {
        tempdir().unwrap()
    }

    #[test]
    fn test_always_condition_with_file() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "test.txt", "content");

        let condition = Always;
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_always_condition_with_directory() {
        let temp_dir = setup_test_env();
        let test_dir = create_temp_dir(temp_dir.path(), "test_directory");

        let condition = Always;
        assert!(!condition.evaluate(&test_dir)); // Directories should return false
    }

    #[test]
    fn test_always_condition_with_nonexistent_path() {
        let temp_dir = setup_test_env();
        let nonexistent_path = temp_dir.path().join("nonexistent.txt");

        let condition = Always;
        assert!(!condition.evaluate(&nonexistent_path)); // Nonexistent files should return false
    }

    #[test]
    fn test_name_equals_exact_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = NameEquals {
            name: "document.pdf".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_equals_no_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = NameEquals {
            name: "other.pdf".to_string(),
        };
        assert!(!condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_equals_case_sensitive() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "Document.PDF", "content");

        let condition = NameEquals {
            name: "document.pdf".to_string(),
        };
        assert!(!condition.evaluate(&test_file)); // Should be case sensitive
    }

    #[test]
    fn test_name_equals_with_unicode() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "测试文件.txt", "content");

        let condition = NameEquals {
            name: "测试文件.txt".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_equals_with_spaces_and_symbols() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(
            temp_dir.path(),
            "file with spaces & symbols!.txt",
            "content",
        );

        let condition = NameEquals {
            name: "file with spaces & symbols!.txt".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_equals_with_invalid_path() {
        let invalid_path = Path::new("");

        let condition = NameEquals {
            name: "test.txt".to_string(),
        };
        assert!(!condition.evaluate(&invalid_path));
    }

    #[test]
    fn test_extension_in_single_extension_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = ExtensionIn {
            extensions: vec!["pdf".to_string()],
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_extension_in_multiple_extensions_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "image.jpg", "content");

        let condition = ExtensionIn {
            extensions: vec!["png".to_string(), "jpg".to_string(), "gif".to_string()],
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_extension_in_no_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = ExtensionIn {
            extensions: vec!["txt".to_string(), "doc".to_string()],
        };
        assert!(!condition.evaluate(&test_file));
    }

    #[test]
    fn test_extension_in_no_extension() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "README", "content");

        let condition = ExtensionIn {
            extensions: vec!["txt".to_string()],
        };
        assert!(!condition.evaluate(&test_file));
    }

    #[test]
    fn test_extension_in_case_sensitive() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.PDF", "content");

        let condition = ExtensionIn {
            extensions: vec!["pdf".to_string()],
        };
        assert!(!condition.evaluate(&test_file)); // Should be case sensitive
    }

    #[test]
    fn test_extension_in_empty_extensions_list() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = ExtensionIn { extensions: vec![] };
        assert!(!condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_contains_substring_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "screenshot_2023.png", "content");

        let condition = NameContains {
            substring: "screenshot".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_contains_no_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = NameContains {
            substring: "screenshot".to_string(),
        };
        assert!(!condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_contains_case_sensitive() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "Screenshot_2023.png", "content");

        let condition = NameContains {
            substring: "screenshot".to_string(),
        };
        assert!(!condition.evaluate(&test_file)); // Should be case sensitive
    }

    #[test]
    fn test_name_contains_partial_match() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "wallpaper_hd.jpg", "content");

        let condition = NameContains {
            substring: "wall".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_contains_empty_substring() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        let condition = NameContains {
            substring: "".to_string(),
        };
        assert!(condition.evaluate(&test_file)); // Empty string should match everything
    }

    #[test]
    fn test_name_contains_with_unicode() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "测试文件_backup.txt", "content");

        let condition = NameContains {
            substring: "测试".to_string(),
        };
        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_name_contains_with_invalid_path() {
        // Create a path that can't be converted to string (this is tricky to simulate)
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "test.txt", "content");

        let condition = NameContains {
            substring: "test".to_string(),
        };
        assert!(condition.evaluate(&test_file)); // Valid path should work
    }

    // Tests for the From implementation
    #[test]
    fn test_from_always_condition() {
        let config_condition = config::Condition {
            condition_type: "always".to_string(),
            value: "".to_string(),
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "test.txt", "content");

        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_from_name_condition() {
        let config_condition = config::Condition {
            condition_type: "name".to_string(),
            value: "document.pdf".to_string(),
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_from_extension_condition_single() {
        let config_condition = config::Condition {
            condition_type: "extension".to_string(),
            value: "pdf".to_string(),
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        assert!(condition.evaluate(&test_file));
    }

    #[test]
    fn test_from_extension_condition_multiple() {
        let config_condition = config::Condition {
            condition_type: "extension".to_string(),
            value: "jpg,png,gif".to_string(),
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let jpg_file = create_temp_file(temp_dir.path(), "image.jpg", "content");
        let png_file = create_temp_file(temp_dir.path(), "image.png", "content");
        let gif_file = create_temp_file(temp_dir.path(), "image.gif", "content");
        let pdf_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        assert!(condition.evaluate(&jpg_file));
        assert!(condition.evaluate(&png_file));
        assert!(condition.evaluate(&gif_file));
        assert!(!condition.evaluate(&pdf_file));
    }

    #[test]
    fn test_from_extension_condition_with_spaces() {
        let config_condition = config::Condition {
            condition_type: "extension".to_string(),
            value: "jpg, png , gif".to_string(), // With spaces
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let jpg_file = create_temp_file(temp_dir.path(), "image.jpg", "content");
        let png_file = create_temp_file(temp_dir.path(), "image.png", "content");

        assert!(condition.evaluate(&jpg_file));
        assert!(condition.evaluate(&png_file));
    }

    #[test]
    fn test_from_name_contains_condition() {
        let config_condition = config::Condition {
            condition_type: "name_contains".to_string(),
            value: "screenshot".to_string(),
        };

        let condition: Box<dyn Condition> = Box::from(&config_condition);

        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "screenshot_2023.png", "content");
        let other_file = create_temp_file(temp_dir.path(), "document.pdf", "content");

        assert!(condition.evaluate(&test_file));
        assert!(!condition.evaluate(&other_file));
    }

    #[test]
    #[should_panic(expected = "Unknown condition type: invalid")]
    fn test_from_unknown_condition_type() {
        let config_condition = config::Condition {
            condition_type: "invalid".to_string(),
            value: "test".to_string(),
        };

        let _condition: Box<dyn Condition> = Box::from(&config_condition);
    }

    // Integration tests
    #[test]
    fn test_multiple_conditions_on_same_file() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "screenshot_wallpaper.jpg", "content");

        let always_condition = Always;
        let name_condition = NameEquals {
            name: "screenshot_wallpaper.jpg".to_string(),
        };
        let extension_condition = ExtensionIn {
            extensions: vec!["jpg".to_string(), "png".to_string()],
        };
        let contains_condition = NameContains {
            substring: "screenshot".to_string(),
        };

        assert!(always_condition.evaluate(&test_file));
        assert!(name_condition.evaluate(&test_file));
        assert!(extension_condition.evaluate(&test_file));
        assert!(contains_condition.evaluate(&test_file));
    }

    #[test]
    fn test_realistic_file_organization_scenario() {
        let temp_dir = setup_test_env();

        // Create various test files
        let music_file = create_temp_file(temp_dir.path(), "song.mp3", "music");
        let image_file = create_temp_file(temp_dir.path(), "photo.jpg", "image");
        let document_file = create_temp_file(temp_dir.path(), "report.pdf", "document");
        let screenshot_file =
            create_temp_file(temp_dir.path(), "screenshot_20231201.png", "screenshot");

        // Test music file organization
        let music_condition = ExtensionIn {
            extensions: vec!["mp3".to_string(), "wav".to_string(), "flac".to_string()],
        };
        assert!(music_condition.evaluate(&music_file));
        assert!(!music_condition.evaluate(&image_file));

        // Test image file organization
        let image_condition = ExtensionIn {
            extensions: vec!["jpg".to_string(), "png".to_string(), "gif".to_string()],
        };
        assert!(image_condition.evaluate(&image_file));
        assert!(image_condition.evaluate(&screenshot_file));
        assert!(!image_condition.evaluate(&document_file));

        // Test screenshot organization
        let screenshot_condition = NameContains {
            substring: "screenshot".to_string(),
        };
        assert!(screenshot_condition.evaluate(&screenshot_file));
        assert!(!screenshot_condition.evaluate(&image_file));

        // Test document organization
        let document_condition = ExtensionIn {
            extensions: vec!["pdf".to_string(), "doc".to_string(), "txt".to_string()],
        };
        assert!(document_condition.evaluate(&document_file));
        assert!(!document_condition.evaluate(&music_file));
    }

    #[test]
    fn test_edge_cases_with_dot_files() {
        let temp_dir = setup_test_env();
        let dot_file = create_temp_file(temp_dir.path(), ".hidden", "content");
        let dot_config = create_temp_file(temp_dir.path(), ".config.json", "content");

        let always_condition = Always;
        let hidden_condition = NameContains {
            substring: ".hidden".to_string(),
        };
        let json_condition = ExtensionIn {
            extensions: vec!["json".to_string()],
        };

        assert!(always_condition.evaluate(&dot_file));
        assert!(always_condition.evaluate(&dot_config));
        assert!(hidden_condition.evaluate(&dot_file));
        assert!(json_condition.evaluate(&dot_config));
    }
}
