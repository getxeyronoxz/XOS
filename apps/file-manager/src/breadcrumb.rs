use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation};

fn home_uri() -> String {
    std::env::var("HOME")
        .map(|home| format!("file://{home}"))
        .unwrap_or_else(|_| "file:///home".to_string())
}

fn uri_path(uri: &str) -> Option<std::path::PathBuf> {
    gio::File::for_uri(uri).path()
}

/// Build a clickable breadcrumb bar for the current directory URI.
pub fn build_breadcrumb_bar(
    current_uri: Rc<RefCell<String>>,
    on_navigate: Rc<dyn Fn(&str)>,
) -> GtkBox {
    let bar = GtkBox::new(Orientation::Horizontal, 4);
    bar.set_margin_start(12);
    bar.set_margin_end(12);
    bar.set_margin_top(6);
    bar.set_margin_bottom(6);
    bar.set_halign(gtk4::Align::Start);

    rebuild_breadcrumb(&bar, &current_uri.borrow(), on_navigate);
    bar
}

pub fn update_breadcrumb(
    bar: &GtkBox,
    uri: &str,
    on_navigate: Rc<dyn Fn(&str)>,
) {
    rebuild_breadcrumb(bar, uri, on_navigate);
}

fn rebuild_breadcrumb(bar: &GtkBox, uri: &str, on_navigate: Rc<dyn Fn(&str)>) {
    while let Some(child) = bar.first_child() {
        bar.remove(&child);
    }

    if uri.starts_with("network:") {
        let label = Label::new(Some("Network"));
        label.add_css_class("dim-label");
        bar.append(&label);
        return;
    }

    let home = home_uri();
    let Some(path) = uri_path(uri) else {
        let label = Label::new(Some(uri));
        label.add_css_class("dim-label");
        bar.append(&label);
        return;
    };

    let home_path = uri_path(&home);
    let mut segments: Vec<(String, String)> = Vec::new();

    if let Some(ref home_pb) = home_path {
        if path.starts_with(home_pb) {
            segments.push(("Home".to_string(), home.clone()));
            if let Ok(relative) = path.strip_prefix(home_pb) {
                let mut accumulated = home_pb.clone();
                for component in relative.components() {
                    if let std::path::Component::Normal(name) = component {
                        accumulated.push(name);
                        let segment_uri = format!("file://{}", accumulated.display());
                        segments.push((name.to_string_lossy().to_string(), segment_uri));
                    }
                }
            }
        }
    }

    if segments.is_empty() {
        let mut accumulated = std::path::PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::RootDir => {
                    accumulated = std::path::PathBuf::from("/");
                    segments.push(("/".to_string(), "file:///".to_string()));
                }
                std::path::Component::Normal(name) => {
                    accumulated.push(name);
                    let segment_uri = format!("file://{}", accumulated.display());
                    segments.push((name.to_string_lossy().to_string(), segment_uri));
                }
                _ => {}
            }
        }
    }

    for (index, (label_text, segment_uri)) in segments.iter().enumerate() {
        if index > 0 {
            let sep = Label::new(Some("/"));
            sep.add_css_class("dim-label");
            bar.append(&sep);
        }

        let btn = Button::with_label(label_text);
        btn.add_css_class("flat");
        let target_uri = segment_uri.clone();
        let navigate = on_navigate.clone();
        btn.connect_clicked(move |_| navigate(&target_uri));
        bar.append(&btn);
    }
}
