use crate::error::CliplyError;
use crate::platform::ImageSnapshot;
use image::imageops::FilterType;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct StoredImageBlob {
    pub image_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub size_bytes: i64,
}

pub fn store_image(
    app: &AppHandle,
    id: &str,
    image: &ImageSnapshot,
) -> Result<StoredImageBlob, CliplyError> {
    let image_dir = blob_dir(app, "images")?;
    let thumbnail_dir = blob_dir(app, "thumbnails")?;

    let image_path = image_dir.join(format!("{id}.{}", image.extension));
    fs::write(&image_path, &image.bytes)?;

    let thumbnail_path = thumbnail_dir.join(format!("{id}.png"));
    write_thumbnail(&image_path, &thumbnail_path)?;

    Ok(StoredImageBlob {
        image_path,
        thumbnail_path,
        size_bytes: image.bytes.len() as i64,
    })
}

fn blob_dir(app: &AppHandle, child: &str) -> Result<PathBuf, CliplyError> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let dir = app_data_dir.join("blobs").join(child);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_thumbnail(image_path: &Path, thumbnail_path: &Path) -> Result<(), CliplyError> {
    let image = image::open(image_path)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let thumbnail = image.resize(360, 260, FilterType::Lanczos3);
    thumbnail
        .save(thumbnail_path)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    Ok(())
}
