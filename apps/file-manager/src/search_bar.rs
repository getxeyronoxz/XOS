use gtk4::prelude::*;
use gtk4::{Box as GtkBox, DropDown, Entry, Orientation, SearchEntry};
use libadwaita as adw;
use adw::prelude::*;

use crate::search::{FileTypeFilter, SearchFilters};

pub struct SearchBar {
    pub widget: adw::PreferencesGroup,
    pub name_entry: SearchEntry,
    pub type_dropdown: DropDown,
    pub min_size_entry: Entry,
    pub max_size_entry: Entry,
    pub modified_days_entry: Entry,
}

impl SearchBar {
    pub fn new() -> Self {
        let group = adw::PreferencesGroup::new();
        group.set_title("Search");
        group.set_description(Some(
            "Filter by name, type, size, or modification date. Searches current folder recursively.",
        ));

        let name_entry = SearchEntry::new();
        name_entry.set_placeholder_text(Some("Search by name…"));
        name_entry.set_margin_start(12);
        name_entry.set_margin_end(12);

        let type_dropdown = DropDown::from_strings(&[
            "All types",
            "Folders",
            "Images",
            "Video",
            "PDF",
            "Code",
        ]);
        let type_row = adw::ActionRow::new();
        type_row.set_title("Type");
        type_row.add_suffix(&type_dropdown);

        let min_size_entry = Entry::new();
        min_size_entry.set_placeholder_text(Some("Min bytes"));
        let max_size_entry = Entry::new();
        max_size_entry.set_placeholder_text(Some("Max bytes"));
        let size_row = adw::ActionRow::new();
        size_row.set_title("Size");
        let size_box = GtkBox::new(Orientation::Horizontal, 8);
        size_box.append(&min_size_entry);
        size_box.append(&max_size_entry);
        size_row.add_suffix(&size_box);

        let modified_days_entry = Entry::new();
        modified_days_entry.set_placeholder_text(Some("Days"));
        let modified_row = adw::ActionRow::new();
        modified_row.set_title("Modified within (days)");
        modified_row.add_suffix(&modified_days_entry);

        group.add(&name_entry);
        group.add(&type_row);
        group.add(&size_row);
        group.add(&modified_row);

        Self {
            widget: group,
            name_entry,
            type_dropdown,
            min_size_entry,
            max_size_entry,
            modified_days_entry,
        }
    }

    pub fn filters(&self) -> SearchFilters {
        let file_type = match self.type_dropdown.selected() {
            1 => FileTypeFilter::Directories,
            2 => FileTypeFilter::Images,
            3 => FileTypeFilter::Video,
            4 => FileTypeFilter::Pdf,
            5 => FileTypeFilter::Code,
            _ => FileTypeFilter::All,
        };

        SearchFilters {
            name: self.name_entry.text().to_string(),
            file_type,
            min_size: parse_optional_u64(&self.min_size_entry.text()),
            max_size: parse_optional_u64(&self.max_size_entry.text()),
            modified_within_days: parse_optional_u32(&self.modified_days_entry.text()),
        }
    }
}

fn parse_optional_u64(value: &str) -> Option<u64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse().ok()
    }
}

fn parse_optional_u32(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        trimmed.parse().ok()
    }
}
