use std::path::Path;

use gio::{self, FileQueryInfoFlags, FileType};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Image, Label, Orientation, Picture, ScrolledWindow, TextView, WrapMode};

const TEXT_PREVIEW_LIMIT: usize = 32_768;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg"];
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "md", "txt", "sh", "bash", "toml", "json", "yaml",
    "yml", "css", "html", "xml", "c", "cpp", "h", "hpp", "go", "java", "rb", "lua",
];
const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm", "avi", "mov", "m4v"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];

pub struct PreviewPanel {
    container: GtkBox,
    title: Label,
    meta: Label,
    content: GtkBox,
}

impl PreviewPanel {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 8);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_width_request(280);

        let title = Label::new(None);
        title.add_css_class("title-4");
        title.set_halign(gtk4::Align::Start);
        title.set_wrap(true);

        let meta = Label::new(None);
        meta.add_css_class("dim-label");
        meta.set_halign(gtk4::Align::Start);
        meta.set_wrap(true);

        let content = GtkBox::new(Orientation::Vertical, 8);

        container.append(&title);
        container.append(&meta);
        container.append(&content);

        Self {
            container,
            title,
            meta,
            content,
        }
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    pub fn clear(&self) {
        self.title.set_text("Preview");
        self.meta.set_text("Select a file to preview");
        clear_box(&self.content);
    }

    pub fn show_uri(&self, uri: &str) {
        clear_box(&self.content);

        let file = gio::File::for_uri(uri);
        let path = match file.path() {
            Some(path) => path,
            None => {
                self.title.set_text("Preview unavailable");
                self.meta.set_text(uri);
                return;
            }
        };

        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("File");
        self.title.set_text(name);

        let info = match file.query_info(
            "standard::size,standard::content-type,standard::type,time::modified",
            gio::FileQueryInfoFlags::NONE,
            gio::Cancellable::NONE,
        ) {
            Ok(info) => info,
            Err(err) => {
                self.meta.set_text(&format!("Error: {err}"));
                return;
            }
        };

        if info.file_type() == gio::FileType::Directory {
            self.meta.set_text("Folder");
            self.content.append(&Label::new(Some("Open to browse contents")));
            return;
        }

        let size = info.size();
        let modified = info
            .modification_date_time()
            .and_then(|dt| dt.format("%Y-%m-%d %H:%M").ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let content_type = info
            .content_type()
            .unwrap_or_else(|| "application/octet-stream".into());

        self.meta.set_text(&format!(
            "{content_type}\n{size} bytes · modified {modified}"
        ));

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();

        if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
            show_image_preview(&self.content, &path);
        } else if CODE_EXTENSIONS.contains(&extension.as_str()) {
            show_text_preview(&self.content, &path, true);
        } else if VIDEO_EXTENSIONS.contains(&extension.as_str()) {
            show_video_preview(&self.content);
        } else if PDF_EXTENSIONS.contains(&extension.as_str()) {
            show_pdf_preview(&self.content);
        } else if is_probably_text(&path) {
            show_text_preview(&self.content, &path, false);
        } else {
            self.content.append(&Label::new(Some(
                "No preview available for this file type.\nDouble-click to open.",
            )));
        }
    }
}

fn show_image_preview(content: &GtkBox, path: &Path) {
    let picture = Picture::new();
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(true);
    picture.set_content_fit(gtk4::ContentFit::Contain);
    picture.set_file(Some(&gio::File::for_path(path)));
    picture.set_size_request(240, 180);
    content.append(&picture);
}

fn show_text_preview(content: &GtkBox, path: &Path, monospace: bool) {
    let text = std::fs::read_to_string(path).unwrap_or_else(|_| {
        std::fs::read(path)
            .map(|bytes| {
                String::from_utf8_lossy(&bytes[..bytes.len().min(TEXT_PREVIEW_LIMIT)]).into_owned()
            })
            .unwrap_or_else(|_| "Unable to read file".to_string())
    });

    let preview = if text.len() > TEXT_PREVIEW_LIMIT {
        format!("{}…", &text[..TEXT_PREVIEW_LIMIT])
    } else {
        text
    };

    let view = TextView::new();
    view.set_editable(false);
    view.set_cursor_visible(false);
    view.set_monospace(monospace);
    view.set_wrap_mode(WrapMode::WordChar);
    view.buffer().set_text(&preview);

    let scrolled = ScrolledWindow::builder()
        .min_content_height(180)
        .child(&view)
        .build();
    content.append(&scrolled);
}

fn show_video_preview(content: &GtkBox) {
    let icon = Image::from_icon_name("video-x-generic-symbolic");
    icon.set_pixel_size(64);
    content.append(&icon);
    content.append(&Label::new(Some(
        "Video preview requires a media backend.\nDouble-click to open in the media player.",
    )));
}

fn show_pdf_preview(content: &GtkBox) {
    let icon = Image::from_icon_name("x-office-document-symbolic");
    icon.set_pixel_size(64);
    content.append(&icon);
    content.append(&Label::new(Some(
        "PDF preview will render inline in a future release.\nDouble-click to open in Evince.",
    )));
}

fn is_probably_text(path: &Path) -> bool {
    let Ok(bytes) = std::fs::read(path) else {
        return false;
    };
    let sample = &bytes[..bytes.len().min(512)];
    !sample.is_empty() && sample.iter().all(|byte| byte.is_ascii() && (*byte == b'\n' || *byte == b'\r' || *byte == b'\t' || *byte >= 0x20))
}

fn clear_box(content: &GtkBox) {
    while let Some(child) = content.first_child() {
        content.remove(&child);
    }
}