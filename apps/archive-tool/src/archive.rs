use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub enum ArchiveFormat {
    Zip,
    TarGz,
}

pub struct ArchiveEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub fn list_archive(archive_path: &Path) -> Result<Vec<ArchiveEntry>, String> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "zip" {
        let file = File::open(archive_path).map_err(|err| format!("Failed to open file: {err}"))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|err| format!("Invalid zip archive: {err}"))?;
        let mut entries = Vec::new();
        for i in 0..zip.len() {
            let file = zip.by_index(i).map_err(|err| format!("Failed to read entry: {err}"))?;
            entries.push(ArchiveEntry {
                name: file.name().to_string(),
                is_dir: file.is_dir(),
                size: file.size(),
            });
        }
        Ok(entries)
    } else if ext == "gz" || ext == "tgz" {
        let file = File::open(archive_path).map_err(|err| format!("Failed to open file: {err}"))?;
        let tar_file = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar_file);
        let mut entries = Vec::new();
        let tar_entries = archive.entries().map_err(|err| format!("Failed to read tar entries: {err}"))?;
        for entry in tar_entries {
            let entry = entry.map_err(|err| format!("Invalid tar entry: {err}"))?;
            let path = entry.path().map_err(|err| format!("Invalid path in tar: {err}"))?;
            let name = path.to_string_lossy().to_string();
            let is_dir = entry.header().entry_type().is_dir();
            let size = entry.header().size().unwrap_or(0);
            entries.push(ArchiveEntry { name, is_dir, size });
        }
        Ok(entries)
    } else {
        Err("Unsupported archive format. Supported formats: .zip, .tar.gz, .tgz".to_string())
    }
}

pub fn extract_archive(archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let ext = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    std::fs::create_dir_all(dest_dir).map_err(|err| format!("Failed to create destination dir: {err}"))?;

    if ext == "zip" {
        let file = File::open(archive_path).map_err(|err| format!("Failed to open file: {err}"))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|err| format!("Invalid zip archive: {err}"))?;
        zip.extract(dest_dir).map_err(|err| format!("Extraction failed: {err}"))?;
        Ok(())
    } else if ext == "gz" || ext == "tgz" {
        let file = File::open(archive_path).map_err(|err| format!("Failed to open file: {err}"))?;
        let tar_file = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(tar_file);
        archive.unpack(dest_dir).map_err(|err| format!("Extraction failed: {err}"))?;
        Ok(())
    } else {
        Err("Unsupported archive format".to_string())
    }
}

pub fn compress_files(files: &[PathBuf], output_archive: &Path, format: ArchiveFormat) -> Result<(), String> {
    if let Some(parent) = output_archive.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("Failed to create output directory: {err}"))?;
    }

    let file = File::create(output_archive).map_err(|err| format!("Failed to create archive file: {err}"))?;

    match format {
        ArchiveFormat::Zip => {
            let mut zip = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            for path in files {
                if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).ok_or("Invalid file name")?;
                    zip.start_file(file_name, options).map_err(|err| format!("Zip write error: {err}"))?;
                    let mut f = File::open(path).map_err(|err| format!("Failed to open file: {err}"))?;
                    io::copy(&mut f, &mut zip).map_err(|err| format!("Failed to copy file contents to zip: {err}"))?;
                } else if path.is_dir() {
                    compress_dir_zip(&mut zip, path, path, options)?;
                }
            }
            zip.finish().map_err(|err| format!("Failed to finalize zip: {err}"))?;
        }
        ArchiveFormat::TarGz => {
            let enc = flate2::write::GzEncoder::new(file, flate2::Compression::default());
            let mut tar = tar::Builder::new(enc);
            for path in files {
                let name = path.file_name().and_then(|n| n.to_str()).ok_or("Invalid file name")?;
                if path.is_file() {
                    let mut f = File::open(path).map_err(|err| format!("Failed to open file: {err}"))?;
                    tar.append_file(name, &mut f).map_err(|err| format!("Tar write error: {err}"))?;
                } else if path.is_dir() {
                    tar.append_dir_all(name, path).map_err(|err| format!("Tar write error: {err}"))?;
                }
            }
            tar.finish().map_err(|err| format!("Failed to finalize tar: {err}"))?;
        }
    }
    Ok(())
}

fn compress_dir_zip<W: io::Write + io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base_dir: &Path,
    current_dir: &Path,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in std::fs::read_dir(current_dir).map_err(|err| format!("Failed to read directory: {err}"))? {
        let entry = entry.map_err(|err| format!("Invalid dir entry: {err}"))?;
        let path = entry.path();
        let name = path.strip_prefix(base_dir).map_err(|_| "Prefix mismatch")?;
        let name_str = name.to_string_lossy().to_string();

        if path.is_file() {
            zip.start_file(&name_str, options).map_err(|err| format!("Zip write error: {err}"))?;
            let mut f = File::open(&path).map_err(|err| format!("Failed to open file: {err}"))?;
            io::copy(&mut f, zip).map_err(|err| format!("Failed to copy to zip: {err}"))?;
        } else if path.is_dir() {
            zip.add_directory(&name_str, options).map_err(|err| format!("Zip write error: {err}"))?;
            compress_dir_zip(zip, base_dir, &path, options)?;
        }
    }
    Ok(())
}
