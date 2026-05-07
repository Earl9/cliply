use crate::error::CliplyError;
use crate::models::settings::CliplyImageSyncSettings;
use crate::platform::ImageSnapshot;
use crate::services::hash_service;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::FilterType;
use image::DynamicImage;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct StoredImageBlob {
    pub image_path: PathBuf,
    pub thumbnail_path: PathBuf,
    pub size_bytes: i64,
}

#[derive(Debug, Clone)]
pub struct PreparedSyncImageBlob {
    pub blob_type: String,
    pub local_path: PathBuf,
    pub size_bytes: i64,
    pub hash: String,
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

pub fn prepare_image_sync_blobs(
    app: &AppHandle,
    id: &str,
    original_path: &Path,
    original_size_bytes: i64,
    settings: &CliplyImageSyncSettings,
) -> Result<Vec<PreparedSyncImageBlob>, CliplyError> {
    let mode = normalized_image_sync_mode(settings.mode.as_str());
    if mode == "metadata-only" {
        return Ok(Vec::new());
    }

    let sync_dir = blob_dir(app, "sync-images")?;
    let max_size_bytes = i64::from(settings.max_image_size_mb.clamp(1, 512)) * 1024 * 1024;
    let mut blobs = Vec::new();

    if matches!(mode, "original" | "original-with-preview") && original_size_bytes <= max_size_bytes
    {
        let original_blob = if settings.strip_metadata {
            let output_path = sync_dir.join(format!("{id}-original.png"));
            write_png_variant(original_path, &output_path, None)?;
            read_prepared_blob("original", output_path)?
        } else {
            read_prepared_blob("original", original_path.to_path_buf())?
        };
        if original_blob.size_bytes <= max_size_bytes {
            blobs.push(original_blob);
        } else if settings.strip_metadata {
            let _ = fs::remove_file(&original_blob.local_path);
        }
    }

    if matches!(mode, "compressed" | "original-with-preview") {
        let blob_type = if mode == "compressed" {
            "compressed"
        } else {
            "preview"
        };
        let output_path = sync_dir.join(format!("{id}-{blob_type}.jpg"));
        write_jpeg_variant(
            original_path,
            &output_path,
            Some(settings.max_dimension.clamp(256, 8192)),
            settings.quality.clamp(40, 95),
        )?;
        let compressed_blob = read_prepared_blob(blob_type, output_path)?;
        if compressed_blob.size_bytes <= max_size_bytes {
            blobs.push(compressed_blob);
        } else {
            let _ = fs::remove_file(&compressed_blob.local_path);
        }
    }

    Ok(blobs)
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

fn normalized_image_sync_mode(mode: &str) -> &'static str {
    match mode {
        "compressed" => "compressed",
        "original" => "original",
        "original-with-preview" => "original-with-preview",
        _ => "metadata-only",
    }
}

fn write_png_variant(
    input_path: &Path,
    output_path: &Path,
    max_dimension: Option<u32>,
) -> Result<(), CliplyError> {
    let image = resized_image(input_path, max_dimension)?;
    image
        .save(output_path)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    Ok(())
}

fn write_jpeg_variant(
    input_path: &Path,
    output_path: &Path,
    max_dimension: Option<u32>,
    quality: u8,
) -> Result<(), CliplyError> {
    let image = resized_image(input_path, max_dimension)?;
    let file = File::create(output_path)?;
    let mut encoder = JpegEncoder::new_with_quality(file, quality);
    encoder
        .encode_image(&image)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    Ok(())
}

fn resized_image(
    input_path: &Path,
    max_dimension: Option<u32>,
) -> Result<DynamicImage, CliplyError> {
    let image = image::open(input_path)
        .map_err(|error| CliplyError::StorageUnavailable(error.to_string()))?;
    let Some(max_dimension) = max_dimension else {
        return Ok(image);
    };

    if image.width() <= max_dimension && image.height() <= max_dimension {
        return Ok(image);
    }

    Ok(image.resize(max_dimension, max_dimension, FilterType::Lanczos3))
}

fn read_prepared_blob(
    blob_type: &str,
    local_path: PathBuf,
) -> Result<PreparedSyncImageBlob, CliplyError> {
    let bytes = fs::read(&local_path)?;
    Ok(PreparedSyncImageBlob {
        blob_type: blob_type.to_string(),
        local_path,
        size_bytes: bytes.len() as i64,
        hash: hash_service::stable_bytes_hash(&bytes),
    })
}
