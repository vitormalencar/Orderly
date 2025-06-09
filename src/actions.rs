use crate::error::OrderlyError;
use crate::Result;
use chrono::prelude::*;
use log::info;
use std::fs;
use std::path::Path;

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

pub fn get_file_name(src_path: &Path) -> Result<String> {
    match src_path.file_name() {
        Some(path) => Ok(path.to_string_lossy().into_owned()),
        None => Err(OrderlyError::InvalidFile(src_path.to_str().map(|s| s.to_string())).into()),
    }
}

pub fn move_file(src_path: &Path, dest_path: &str) -> Result<()> {
    let mut dest_path = Path::new(dest_path).to_path_buf();

    // Ensure the destination directory exists
    ensure_directory_exists(&dest_path)?;

    // Append the file name to the destination path if it's a directory
    if dest_path.is_dir() {
        dest_path = dest_path.join(get_file_name(src_path)?);
    }
    // Try moving the file, falling back to copy and remove if necessary
    move_or_fallback(src_path, &dest_path)
}

pub fn delete_file(path: &Path) -> Result<()> {
    trash::delete(path).map_err(OrderlyError::TrashError)?;
    info!("Deleted file {}", path.display());
    Ok(())
}

pub fn copy_file(src_path: &Path, dest: &str) -> Result<()> {
    let mut dest_path = Path::new(dest).to_path_buf();

    ensure_directory_exists(&dest_path)?;

    // Append the file name to the destination path if it's a directory
    if dest_path.is_dir() {
        dest_path = dest_path.join(get_file_name(src_path)?);
    }
    // Copy
    fs::copy(src_path, &dest_path)?;
    info!(
        "Copied file from {} to {}",
        src_path.display(),
        dest_path.display()
    );
    Ok(())
}

pub fn sort_file_by_date(src_path: &Path, base_dest: &str, pattern: &str) -> Result<()> {
    let metadata = fs::metadata(src_path)?;
    let modified_time = metadata.modified()?;
    let datetime: DateTime<Local> = modified_time.into();

    let formatted_date = datetime.format(pattern).to_string();
    let dest_path = Path::new(base_dest).join(formatted_date);

    ensure_directory_exists(&dest_path)?;

    let final_dest = dest_path.join(get_file_name(src_path)?);
    move_or_fallback(src_path, &final_dest)
}
