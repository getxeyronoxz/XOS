use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gio::{self, File, FileQueryInfoFlags, FileType};
use gio::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub name: String,
    pub file_type: FileTypeFilter,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub modified_within_days: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FileTypeFilter {
    #[default]
    All,
    Directories,
    Images,
    Video,
    Pdf,
    Code,
}

#[derive(Debug, Clone)]
pub struct FileItem {
    pub uri: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
    pub content_type: String,
}

impl SearchFilters {
    pub fn is_active(&self) -> bool {
        !self.name.is_empty()
            || self.file_type != FileTypeFilter::All
            || self.min_size.is_some()
            || self.max_size.is_some()
            || self.modified_within_days.is_some()
    }
}

pub fn search_directory(root_uri: &str, filters: &SearchFilters) -> Result<Vec<FileItem>, String> {
    let root = File::for_uri(root_uri);
    let mut results = Vec::new();
    walk_directory(&root, filters, &mut results, 0)?;
    results.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(results)
}

fn walk_directory(
    directory: &File,
    filters: &SearchFilters,
    results: &mut Vec<FileItem>,
    depth: u8,
) -> Result<(), String> {
    if depth > 8 {
        return Ok(());
    }

    let enumerator = directory
        .enumerate_children(
            "standard::display-name,standard::type,standard::size,standard::content-type,time::modified",
            FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        )
        .map_err(|err| format!("Search failed: {err}"))?;

    loop {
        let info = match enumerator.next_file(gio::Cancellable::NONE) {
            Ok(Some(info)) => info,
            Ok(None) => break,
            Err(err) => return Err(format!("Search failed: {err}")),
        };

        let name = info.display_name().to_string();
        if name == "." || name == ".." {
            continue;
        }

        let child = directory.child(name.as_str());
        let uri = child.uri().to_string();
        let is_dir = info.file_type() == FileType::Directory;
        let size = if is_dir { 0 } else { info.size() as u64 };
        let modified = info
            .modification_time()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let content_type = info
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let item = FileItem {
            uri,
            name: name.clone(),
            is_dir,
            size,
            modified,
            content_type,
        };

        if matches_filters(&item, filters) {
            results.push(item.clone());
        }

        if is_dir {
            walk_directory(&child, filters, results, depth + 1)?;
        }
    }

    Ok(())
}

fn matches_filters(item: &FileItem, filters: &SearchFilters) -> bool {
    if !filters.name.is_empty()
        && !item
            .name
            .to_ascii_lowercase()
            .contains(&filters.name.to_ascii_lowercase())
    {
        return false;
    }

    match filters.file_type {
        FileTypeFilter::All => {}
        FileTypeFilter::Directories => {
            if !item.is_dir {
                return false;
            }
        }
        FileTypeFilter::Images => {
            if item.is_dir || !item.content_type.starts_with("image/") {
                return false;
            }
        }
        FileTypeFilter::Video => {
            if item.is_dir || !item.content_type.starts_with("video/") {
                return false;
            }
        }
        FileTypeFilter::Pdf => {
            if item.is_dir || item.content_type != "application/pdf" {
                return false;
            }
        }
        FileTypeFilter::Code => {
            if item.is_dir {
                return false;
            }
            let ext = item
                .name
                .rsplit('.')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if !matches!(
                ext.as_str(),
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "tsx"
                    | "jsx"
                    | "md"
                    | "txt"
                    | "sh"
                    | "toml"
                    | "json"
                    | "yaml"
                    | "yml"
                    | "css"
                    | "html"
                    | "c"
                    | "cpp"
                    | "go"
            ) {
                return false;
            }
        }
    }

    if let Some(min_size) = filters.min_size {
        if item.size < min_size {
            return false;
        }
    }

    if let Some(max_size) = filters.max_size {
        if item.size > max_size {
            return false;
        }
    }

    if let Some(days) = filters.modified_within_days {
        let cutoff = SystemTime::now()
            .checked_sub(Duration::from_secs(u64::from(days) * 86_400))
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0);
        if item.modified < cutoff {
            return false;
        }
    }

    true
}
