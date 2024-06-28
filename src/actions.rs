pub fn move_file(path: &str, destination: &str) {
  std::fs::rename(path, destination).expect("Failed to move file");
}

// Add more actions like copy, delete, etc.
