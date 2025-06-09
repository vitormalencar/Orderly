#[cfg(test)]
mod tests {
    use crate::actions::*;
    use crate::error::OrderlyError;
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

    // Helper function to create a temporary directory structure
    fn setup_test_env() -> TempDir {
        tempdir().unwrap()
    }

    #[test]
    fn test_get_file_name_valid_file() {
        let path = Path::new("/path/to/test_file.txt");
        let result = get_file_name(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_file.txt");
    }

    #[test]
    fn test_get_file_name_with_extension() {
        let path = Path::new("/path/to/document.pdf");
        let result = get_file_name(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "document.pdf");
    }

    #[test]
    fn test_get_file_name_no_extension() {
        let path = Path::new("/path/to/README");
        let result = get_file_name(&path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "README");
    }

    #[test]
    fn test_get_file_name_root_path() {
        let path = Path::new("/");
        let result = get_file_name(&path);
        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<OrderlyError>() {
            Some(OrderlyError::InvalidFile(_)) => (),
            _ => panic!("Expected InvalidFile error"),
        }
    }

    #[test]
    fn test_get_file_name_empty_path() {
        let path = Path::new("");
        let result = get_file_name(&path);
        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<OrderlyError>() {
            Some(OrderlyError::InvalidFile(_)) => (),
            _ => panic!("Expected InvalidFile error"),
        }
    }

    #[test]
    fn test_move_file_to_directory() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_dir = temp_dir.path().join("destination");
        fs::create_dir_all(&dest_dir).unwrap();

        let result = move_file(&src_file, dest_dir.to_str().unwrap());

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved
        assert!(dest_dir.join("source.txt").exists()); // Destination should exist
    }

    #[test]
    fn test_move_file_to_new_directory() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_dir = temp_dir.path().join("new_destination");

        let result = move_file(&src_file, dest_dir.to_str().unwrap());

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved
        assert!(dest_dir.exists()); // Destination directory should be created
        assert!(dest_dir.join("source.txt").exists()); // File should exist in destination
    }

    #[test]
    fn test_move_file_with_specific_filename() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_file = temp_dir.path().join("destination.txt");

        let result = move_file(&src_file, dest_file.to_str().unwrap());

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved
        assert!(dest_file.exists()); // Destination file should exist
    }

    #[test]
    fn test_move_file_nonexistent_source() {
        let temp_dir = setup_test_env();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");
        let dest_dir = temp_dir.path().join("destination");

        let result = move_file(&nonexistent_file, dest_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_copy_file_to_directory() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_dir = temp_dir.path().join("destination");
        fs::create_dir_all(&dest_dir).unwrap();

        let result = copy_file(&src_file, dest_dir.to_str().unwrap());

        assert!(result.is_ok());
        assert!(src_file.exists()); // Source should still exist
        assert!(dest_dir.join("source.txt").exists()); // Copy should exist in destination

        // Verify content is the same
        let original_content = fs::read_to_string(&src_file).unwrap();
        let copied_content = fs::read_to_string(dest_dir.join("source.txt")).unwrap();
        assert_eq!(original_content, copied_content);
    }

    #[test]
    fn test_copy_file_to_new_directory() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_dir = temp_dir.path().join("new_destination");

        let result = copy_file(&src_file, dest_dir.to_str().unwrap());

        assert!(result.is_ok());
        assert!(src_file.exists()); // Source should still exist
        assert!(dest_dir.exists()); // Destination directory should be created
        assert!(dest_dir.join("source.txt").exists()); // Copy should exist
    }

    #[test]
    fn test_copy_file_with_specific_filename() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "source.txt", "test content");
        let dest_file = temp_dir.path().join("destination.txt");

        let result = copy_file(&src_file, dest_file.to_str().unwrap());

        assert!(result.is_ok());
        assert!(src_file.exists()); // Source should still exist
        assert!(dest_file.exists()); // Destination file should exist
    }

    #[test]
    fn test_copy_file_nonexistent_source() {
        let temp_dir = setup_test_env();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");
        let dest_dir = temp_dir.path().join("destination");

        let result = copy_file(&nonexistent_file, dest_dir.to_str().unwrap());

        assert!(result.is_err());
    }

    #[test]
    fn test_delete_file_existing() {
        let temp_dir = setup_test_env();
        let test_file = create_temp_file(temp_dir.path(), "to_delete.txt", "content");

        assert!(test_file.exists());

        let result = delete_file(&test_file);

        assert!(result.is_ok());
        // Note: The file might still exist temporarily due to trash system
        // We mainly check that the operation completed without error
    }

    #[test]
    fn test_delete_file_nonexistent() {
        let temp_dir = setup_test_env();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        let result = delete_file(&nonexistent_file);

        assert!(result.is_err());
    }

    #[test]
    fn test_sort_file_by_date_year_pattern() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "document.txt", "test content");
        let base_dest = temp_dir.path().join("sorted");

        let result = sort_file_by_date(&src_file, base_dest.to_str().unwrap(), "%Y");

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved

        // Check that a directory with current year was created
        let year = chrono::Local::now().format("%Y").to_string();
        let year_dir = base_dest.join(year);
        assert!(year_dir.exists());
        assert!(year_dir.join("document.txt").exists());
    }

    #[test]
    fn test_sort_file_by_date_year_month_pattern() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "photo.jpg", "image data");
        let base_dest = temp_dir.path().join("photos");

        let result = sort_file_by_date(&src_file, base_dest.to_str().unwrap(), "%Y-%m");

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved

        // Check that a directory with current year-month was created
        let year_month = chrono::Local::now().format("%Y-%m").to_string();
        let date_dir = base_dest.join(year_month);
        assert!(date_dir.exists());
        assert!(date_dir.join("photo.jpg").exists());
    }

    #[test]
    fn test_sort_file_by_date_full_date_pattern() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "report.pdf", "report content");
        let base_dest = temp_dir.path().join("reports");

        let result = sort_file_by_date(&src_file, base_dest.to_str().unwrap(), "%Y-%m-%d");

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved

        // Check that a directory with current date was created
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let date_dir = base_dest.join(date);
        assert!(date_dir.exists());
        assert!(date_dir.join("report.pdf").exists());
    }

    #[test]
    fn test_sort_file_by_date_nonexistent_file() {
        let temp_dir = setup_test_env();
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");
        let base_dest = temp_dir.path().join("sorted");

        let result = sort_file_by_date(&nonexistent_file, base_dest.to_str().unwrap(), "%Y");

        assert!(result.is_err());
    }

    #[test]
    fn test_sort_file_by_date_custom_pattern() {
        let temp_dir = setup_test_env();
        let src_file = create_temp_file(temp_dir.path(), "backup.zip", "backup data");
        let base_dest = temp_dir.path().join("backups");

        let result = sort_file_by_date(&src_file, base_dest.to_str().unwrap(), "%B_%Y");

        assert!(result.is_ok());
        assert!(!src_file.exists()); // Source should be moved

        // Check that a directory with month_year format was created
        let month_year = chrono::Local::now().format("%B_%Y").to_string();
        let date_dir = base_dest.join(month_year);
        assert!(date_dir.exists());
        assert!(date_dir.join("backup.zip").exists());
    }

    // Integration tests for combinations of operations
    #[test]
    fn test_multiple_file_operations() {
        let temp_dir = setup_test_env();

        // Create multiple test files
        let file1 = create_temp_file(temp_dir.path(), "file1.txt", "content1");
        let file2 = create_temp_file(temp_dir.path(), "file2.txt", "content2");
        let file3 = create_temp_file(temp_dir.path(), "file3.txt", "content3");

        let dest_dir = temp_dir.path().join("organized");

        // Test multiple copy operations
        assert!(copy_file(&file1, dest_dir.to_str().unwrap()).is_ok());
        assert!(copy_file(&file2, dest_dir.to_str().unwrap()).is_ok());
        assert!(copy_file(&file3, dest_dir.to_str().unwrap()).is_ok());

        // Verify all files were copied
        assert!(file1.exists() && dest_dir.join("file1.txt").exists());
        assert!(file2.exists() && dest_dir.join("file2.txt").exists());
        assert!(file3.exists() && dest_dir.join("file3.txt").exists());

        // Test moving the original files to another location
        let archive_dir = temp_dir.path().join("archive");
        assert!(move_file(&file1, archive_dir.to_str().unwrap()).is_ok());
        assert!(move_file(&file2, archive_dir.to_str().unwrap()).is_ok());
        assert!(move_file(&file3, archive_dir.to_str().unwrap()).is_ok());

        // Verify files were moved
        assert!(!file1.exists() && archive_dir.join("file1.txt").exists());
        assert!(!file2.exists() && archive_dir.join("file2.txt").exists());
        assert!(!file3.exists() && archive_dir.join("file3.txt").exists());
    }

    #[test]
    fn test_edge_case_unicode_filename() {
        let temp_dir = setup_test_env();
        let unicode_file = create_temp_file(temp_dir.path(), "测试文件.txt", "unicode content");

        let result = get_file_name(&unicode_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "测试文件.txt");

        let dest_dir = temp_dir.path().join("unicode_dest");
        let copy_result = copy_file(&unicode_file, dest_dir.to_str().unwrap());
        assert!(copy_result.is_ok());
        assert!(dest_dir.join("测试文件.txt").exists());
    }

    #[test]
    fn test_edge_case_special_characters_filename() {
        let temp_dir = setup_test_env();
        let special_file = create_temp_file(
            temp_dir.path(),
            "file with spaces & symbols!.txt",
            "special content",
        );

        let result = get_file_name(&special_file);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file with spaces & symbols!.txt");

        let dest_dir = temp_dir.path().join("special_dest");
        let copy_result = copy_file(&special_file, dest_dir.to_str().unwrap());
        assert!(copy_result.is_ok());
        assert!(dest_dir.join("file with spaces & symbols!.txt").exists());
    }
}
