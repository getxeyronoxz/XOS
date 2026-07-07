use std::path::{Path, PathBuf};

use crate::error::NotesError;

pub struct NoteEntry {
    pub id: String,
    pub title: String,
    pub path: PathBuf,
}

pub fn notes_dir() -> Result<PathBuf, NotesError> {
    let home = std::env::var("HOME").map_err(|_| NotesError::Storage("HOME is not set".into()))?;
    Ok(Path::new(&home).join(".local/share/xos/notes"))
}

pub fn list_notes() -> Result<Vec<NoteEntry>, NotesError> {
    let dir = notes_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|err| NotesError::Storage(format!("Failed to create notes dir: {err}")))?;

    let mut notes = Vec::new();
    let mut entries: Vec<_> = std::fs::read_dir(&dir)
        .map_err(|err| NotesError::Storage(format!("Failed to read notes dir: {err}")))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let id = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("note")
            .to_string();
        let title = read_title(&path).unwrap_or_else(|| id.clone());
        notes.push(NoteEntry { id, title, path });
    }

    Ok(notes)
}

pub fn read_note(path: &Path) -> Result<String, NotesError> {
    std::fs::read_to_string(path)
        .map_err(|err| NotesError::Storage(format!("Failed to read {}: {err}", path.display())))
}

pub fn write_note(path: &Path, content: &str) -> Result<(), NotesError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| NotesError::Storage(format!("Failed to create notes dir: {err}")))?;
    }
    std::fs::write(path, content)
        .map_err(|err| NotesError::Storage(format!("Failed to write {}: {err}", path.display())))
}

pub fn create_note() -> Result<NoteEntry, NotesError> {
    let dir = notes_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|err| NotesError::Storage(format!("Failed to create notes dir: {err}")))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let id = format!("note-{timestamp}");
    let path = dir.join(format!("{id}.md"));
    let content = "# New Note\n\n";
    write_note(&path, content)?;
    Ok(NoteEntry {
        id: id.clone(),
        title: "New Note".to_string(),
        path,
    })
}

pub fn delete_note(path: &Path) -> Result<(), NotesError> {
    std::fs::remove_file(path)
        .map_err(|err| NotesError::Storage(format!("Failed to delete {}: {err}", path.display())))
}

fn read_title(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line.trim_start_matches("# ").trim().to_string())
        .filter(|title| !title.is_empty())
}
