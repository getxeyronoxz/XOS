use gio::{self, FileCopyFlags, FileType};
use gio::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOperation {
    Copy,
    Move,
}

#[derive(Debug, Clone)]
pub struct ClipboardEntry {
    pub source_uri: String,
    pub operation: ClipboardOperation,
}

pub fn copy_into_directory(source_uri: &str, dest_dir_uri: &str) -> Result<(), String> {
    let source = gio::File::for_uri(source_uri);
    let dest_dir = gio::File::for_uri(dest_dir_uri);
    let file_name = source
        .basename()
        .ok_or_else(|| format!("Cannot determine name for {source_uri}"))?;
    let file_name_str = file_name.to_str().ok_or_else(|| "Invalid file name".to_string())?;
    let destination = unique_destination(&dest_dir, file_name_str)?;

    source
        .copy(
            &destination,
            FileCopyFlags::NONE,
            None::<&gio::Cancellable>,
            None::<&mut dyn FnMut(i64, i64)>,
        )
        .map_err(|err| format!("Copy failed: {err}"))
}

pub fn move_into_directory(source_uri: &str, dest_dir_uri: &str) -> Result<(), String> {
    let source = gio::File::for_uri(source_uri);
    let dest_dir = gio::File::for_uri(dest_dir_uri);
    let file_name = source
        .basename()
        .ok_or_else(|| format!("Cannot determine name for {source_uri}"))?;
    let file_name_str = file_name.to_str().ok_or_else(|| "Invalid file name".to_string())?;
    let destination = unique_destination(&dest_dir, file_name_str)?;

    source
        .move_(
            &destination,
            FileCopyFlags::NONE,
            None::<&gio::Cancellable>,
            None::<&mut dyn FnMut(i64, i64)>,
        )
        .map_err(|err| format!("Move failed: {err}"))
}

pub fn trash_file(source_uri: &str) -> Result<(), String> {
    let source = gio::File::for_uri(source_uri);
    source
        .trash(gio::Cancellable::NONE)
        .map_err(|err| format!("Delete failed: {err}"))
}

pub fn parent_uri(uri: &str) -> Option<String> {
    let file = gio::File::for_uri(uri);
    file.parent().map(|parent| parent.uri().to_string())
}

pub fn is_directory(uri: &str) -> bool {
    gio::File::for_uri(uri)
        .query_file_type(gio::FileQueryInfoFlags::NONE, None::<&gio::Cancellable>) == FileType::Directory
}

fn unique_destination(directory: &gio::File, file_name: &str) -> Result<gio::File, String> {
    let mut candidate = directory.child(file_name);
    if !candidate.query_exists(gio::Cancellable::NONE) {
        return Ok(candidate);
    }

    let stem = std::path::Path::new(file_name)
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string());
    let extension = std::path::Path::new(file_name)
        .extension()
        .map(|value| value.to_string_lossy().into_owned());

    for index in 1..=999 {
        let next_name = match extension.as_deref() {
            Some(ext) => format!("{stem} ({index}).{ext}"),
            None => format!("{stem} ({index})"),
        };
        candidate = directory.child(next_name.as_str());
        if !candidate.query_exists(gio::Cancellable::NONE) {
            return Ok(candidate);
        }
    }

    Err(format!("Could not find available name for {file_name}"))
}
