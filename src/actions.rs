use log::info;
use std::fs;
use std::io::{self, Result};
use std::path::Path;
use trash;

pub fn move_file(src: &str, dest: &str) -> Result<()> {
    let src_path = Path::new(src);
    let dest_path = Path::new(dest);

    // Ensure the destination directory exists
    if dest_path.is_dir() {
        fs::create_dir_all(dest_path)?;
        info!("Created directory: {}", dest_path.display());
    } else if let Some(parent) = dest_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
            info!("Created directory: {}", parent.display());
        }
    }

    // Append the file name to the destination path if it's a directory
    let final_dest = if dest_path.is_dir() {
        dest_path.join(src_path.file_name().unwrap())
    } else {
        dest_path.to_path_buf()
    };

    // Try renaming the file, if it fails, fallback to copy and remove
    match fs::rename(src_path, &final_dest) {
        Ok(_) => {
            info!("Moved file from {} to {}", src, final_dest.display());
            Ok(())
        }
        Err(e) => {
            // Fallback to copy and remove if rename fails
            fs::copy(src_path, &final_dest)?;
            fs::remove_file(src_path)?;
            info!(
                "Copied and removed file from {} to {}",
                src,
                final_dest.display()
            );
            Err(e)
        }
    }
}

pub fn delete_file(path: &str) -> Result<()> {
    // Use trash::delete to move the file to the trash, handling the error conversion
    trash::delete(Path::new(path))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    info!("Deleted file {}", path);
    Ok(())
}
