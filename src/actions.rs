use log::info;
use std::fs;
use std::io::{self, Result};
use std::path::Path;
use trash;

fn ensure_directory_exists(dest_path: &Path) -> Result<()> {
    if !dest_path.exists() {
        fs::create_dir_all(dest_path)?;
        info!("Created directory: {}", dest_path.display());
    }

    Ok(())
}

fn move_or_fallback(src: &Path, dest: &Path) -> Result<()> {
    match fs::rename(src, dest) {
        Ok(_) => {
            info!("Moved file from {} to {}", src.display(), dest.display());
            Ok(())
        }
        Err(_) => {
            // Fallback to copy and remove if rename fails
            fs::copy(src, dest)?;
            fs::remove_file(src)?;
            info!(
                "Copied and removed file from {} to {}",
                src.display(),
                dest.display()
            );
            Ok(())
        }
    }
}

pub fn move_file(src: &str, dest: &str) -> Result<()> {
    let src_path = Path::new(src);
    let mut dest_path = Path::new(dest).to_path_buf();

    // Frist check if the destination path exists
    ensure_directory_exists(&dest_path)?;

    // Append the file name to the destination path if it's a directory
    if dest_path.is_dir() {
        dest_path = dest_path.join(src_path.file_name().unwrap());
    }
    // Try moving the file, falling back to copy and remove if necessary
    move_or_fallback(src_path, &dest_path)
}

pub fn delete_file(path: &str) -> Result<()> {
    trash::delete(Path::new(path))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    info!("Deleted file {}", path);
    Ok(())
}

pub fn copy_file(src: &str, dest: &str) -> Result<()> {
    let src_path = Path::new(src);
    let mut dest_path = Path::new(dest).to_path_buf();

    ensure_directory_exists(&dest_path)?;

    // Append the file name to the destination path if it's a directory
    if dest_path.is_dir() {
        dest_path = dest_path.join(src_path.file_name().unwrap());
    }
    // copy
    fs::copy(src_path, &dest_path)?;
    info!(
        "Copied file from {} to {}",
        src_path.display(),
        dest_path.display()
    );
    Ok(())
}
