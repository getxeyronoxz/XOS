use gtk4::prelude::*;
use gtk4::{gio, Label, ListBox, ListBoxRow, ScrolledWindow, SelectionMode};

fn home_uri() -> String {
    std::env::var("HOME")
        .map(|home| format!("file://{home}"))
        .unwrap_or_else(|_| "file:///home".to_string())
}

fn bookmark_uri(subpath: &str) -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|home| format!("file://{home}/{subpath}"))
}

/// Sidebar with bookmarks, drives, and network places.
pub fn build_sidebar(on_navigate: std::rc::Rc<dyn Fn(&str)>) -> ScrolledWindow {
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::None);
    list_box.add_css_class("navigation-sidebar");

    append_section_header(&list_box, "Bookmarks");
    append_nav_row(&list_box, "Home", &home_uri(), &on_navigate);
    if let Some(desktop) = bookmark_uri("Desktop") {
        append_nav_row(&list_box, "Desktop", &desktop, &on_navigate);
    }
    if let Some(documents) = bookmark_uri("Documents") {
        append_nav_row(&list_box, "Documents", &documents, &on_navigate);
    }
    if let Some(downloads) = bookmark_uri("Downloads") {
        append_nav_row(&list_box, "Downloads", &downloads, &on_navigate);
    }

    append_section_header(&list_box, "Drives");
    populate_drives(&list_box, &on_navigate);

    append_section_header(&list_box, "Network");
    append_nav_row(&list_box, "Network Places", "network:///", &on_navigate);

    let monitor = gio::VolumeMonitor::get();
    let list_for_mounts = list_box.clone();
    let navigate = on_navigate.clone();
    monitor.connect_mount_added(move |_, mount| {
        let root = mount.root();
        let name = mount.name().to_string();
        let uri = root.uri().to_string();
        append_nav_row(&list_for_mounts, &name, &uri, &navigate);
    });

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .min_content_width(200)
        .child(&list_box)
        .build();

    scrolled
}

fn append_section_header(list_box: &ListBox, title: &str) {
    let label = Label::new(Some(title));
    label.add_css_class("heading");
    label.set_margin_start(12);
    label.set_margin_end(12);
    label.set_margin_top(12);
    label.set_margin_bottom(4);
    label.set_halign(gtk4::Align::Start);

    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);
    row.set_child(Some(&label));
    list_box.append(&row);
}

fn append_nav_row(list_box: &ListBox, label_text: &str, uri: &str, on_navigate: &std::rc::Rc<dyn Fn(&str)>) {
    let label = Label::new(Some(label_text));
    label.set_xalign(0.0);
    label.set_margin_start(12);
    label.set_margin_end(12);
    label.set_margin_top(6);
    label.set_margin_bottom(6);

    let row = ListBoxRow::new();
    row.set_widget_name(&format!("nav:{uri}"));
    row.set_child(Some(&label));

    let target = uri.to_string();
    let navigate = on_navigate.clone();
    row.connect_activate(move |_| navigate(&target));

    list_box.append(&row);
}

fn populate_drives(list_box: &ListBox, on_navigate: &std::rc::Rc<dyn Fn(&str)>) {
    let monitor = gio::VolumeMonitor::get();
    let mut found = false;

    for mount in monitor.mounts() {
        let root = mount.root();
        let name = mount.name().to_string();
        let uri = root.uri().to_string();
        append_nav_row(list_box, &name, &uri, on_navigate);
        found = true;
    }

    if !found {
        let label = Label::new(Some("No drives mounted"));
        label.add_css_class("dim-label");
        label.set_margin_start(12);
        label.set_margin_end(12);

        let row = ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);
        row.set_child(Some(&label));
        list_box.append(&row);
    }
}
