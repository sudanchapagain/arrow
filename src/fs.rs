use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn prepare_directories(dist_dir: &Path) -> Result<()> {
    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir).context("failed to clear destination directory")?;
    }
    fs::create_dir_all(dist_dir).context("failed to create destination directory")?;
    Ok(())
}

pub fn collect_djot_files(src_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "djot") {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

pub fn copy_assets(src: &Path, dest: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative_path = path.strip_prefix(src).context("failed to strip prefix")?;
        let dest_path = dest.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&dest_path).context("failed to create asset directory")?;
        } else if path.is_file() {
            fs::copy(path, &dest_path).context("failed to copy asset file")?;
        }
    }
    Ok(())
}
