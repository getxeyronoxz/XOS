use gtk4::prelude::*;
use gtk4::{Label, ListBox, ListBoxRow};
use gio::{self, FileQueryInfoFlags, FileType};

use crate::search::FileItem;

pub fn populate_directory(list_box: &ListBox, uri: &str) {
    while let Some(row) = list_box.row_at_index(0) {
        list_box.remove(&row);
    }

    if uri.starts_with("network:") {
        append_message_row(
            list_box,
            "Network places — smb:// and other URIs coming in a future release",
        );
        return;
    }

    let file = gio::File::for_uri(uri);
    let enumerator = match file.enumerate_children(
        "standard::display-name,standard::type",
        FileQueryInfoFlags::NONE,
        gio::Cancellable::NONE,
    ) {
        Ok(e) => e,
        Err(err) => {
            append_message_row(list_box, &format!("Error: {err}"));
            return;
        }
    };

    let mut items = Vec::new();

    loop {
        let info = match enumerator.next_file(gio::Cancellable::NONE) {
            Ok(Some(info)) => info,
            Ok(None) => break,
            Err(err) => {
                append_message_row(list_box, &format!("Error: {err}"));
                break;
            }
        };

        let name = info.display_name().to_string();
        let child = file.child(name.as_str());
        let is_dir = info.file_type() == FileType::Directory;
        items.push(FileItem {
            uri: child.uri().to_string(),
            name,
            is_dir,
            size: 0,
            modified: 0,
            content_type: String::new(),
        });
    }

    populate_items(list_box, &items);
}

pub fn populate_items(list_box: &ListBox, items: &[FileItem]) {
    while let Some(row) = list_box.row_at_index(0) {
        list_box.remove(&row);
    }

    if items.is_empty() {
        append_message_row(list_box, "No matching files");
        return;
    }

    for item in items {
        let display = if item.is_dir {
            format!("📁 {}", item.name)
        } else {
            format!("📄 {}", item.name)
        };

        let label = Label::new(Some(&display));
        label.set_xalign(0.0);
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(6);
        label.set_margin_bottom(6);

        let row = ListBoxRow::new();
        if item.is_dir {
            row.set_widget_name(&format!("dir:{}", item.uri));
        } else {
            row.set_widget_name(&item.uri);
        }
        row.set_child(Some(&label));
        list_box.append(&row);
    }
}

pub fn append_message_row(list_box: &ListBox, message: &str) {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);
    row.set_child(Some(&Label::new(Some(message))));
    list_box.append(&row);
}
