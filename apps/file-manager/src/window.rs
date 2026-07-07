use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Paned, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::breadcrumb::{build_breadcrumb_bar, update_breadcrumb};
use crate::listing::{append_message_row, populate_directory, populate_items};
use crate::operations::{
    copy_into_directory, is_directory, move_into_directory, parent_uri, trash_file,
    ClipboardEntry, ClipboardOperation,
};
use crate::preview::PreviewPanel;
use crate::search::search_directory;
use crate::search_bar::SearchBar;
use crate::sidebar::build_sidebar;

fn home_uri() -> String {
    std::env::var("HOME")
        .map(|home| format!("file://{home}"))
        .unwrap_or_else(|_| "file:///home".to_string())
}

struct BrowserState {
    current_uri: RefCell<String>,
    clipboard: RefCell<Option<ClipboardEntry>>,
}

impl BrowserState {
    fn new(initial_uri: String) -> Rc<Self> {
        Rc::new(Self {
            current_uri: RefCell::new(initial_uri),
            clipboard: RefCell::new(None),
        })
    }

    fn current_uri(&self) -> String {
        self.current_uri.borrow().clone()
    }

    fn set_current_uri(&self, uri: String) {
        *self.current_uri.borrow_mut() = uri;
    }
}

struct UiContext {
    state: Rc<BrowserState>,
    list_box: gtk4::ListBox,
    breadcrumb_bar: GtkBox,
    breadcrumb_uri: Rc<RefCell<String>>,
    search_bar: SearchBar,
    preview: PreviewPanel,
}

impl UiContext {
    fn refresh(self: &Rc<Self>, navigate: Rc<dyn Fn(&str)>) {
        let uri = self.state.current_uri();
        *self.breadcrumb_uri.borrow_mut() = uri.clone();
        update_breadcrumb(&self.breadcrumb_bar, &uri, navigate);

        let filters = self.search_bar.filters();
        if filters.is_active() {
            match search_directory(&uri, &filters) {
                Ok(items) => populate_items(&self.list_box, &items),
                Err(err) => {
                    while let Some(row) = self.list_box.row_at_index(0) {
                        self.list_box.remove(&row);
                    }
                    append_message_row(&self.list_box, &err);
                }
            }
        } else {
            populate_directory(&self.list_box, &uri);
        }

        self.preview.clear();
    }
}

pub struct FileManagerWindow {
    window: adw::ApplicationWindow,
}

impl FileManagerWindow {
    pub fn new(app: &adw::Application) -> Self {
        let state = BrowserState::new(home_uri());

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("XOS Files")
            .default_width(1180)
            .default_height(680)
            .build();

        let toast_overlay = adw::ToastOverlay::new();
        window.set_content(Some(&toast_overlay));

        let header = adw::HeaderBar::new();
        let up_button = Button::with_label("Up");
        let copy_button = Button::with_label("Copy");
        let cut_button = Button::with_label("Cut");
        let paste_button = Button::with_label("Paste");
        let delete_button = Button::with_label("Delete");

        header.pack_start(&up_button);
        header.pack_end(&delete_button);
        header.pack_end(&paste_button);
        header.pack_end(&cut_button);
        header.pack_end(&copy_button);

        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);

        let file_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_box)
            .build();

        let preview = PreviewPanel::new();
        let preview_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_width(280)
            .child(preview.widget())
            .build();

        let list_preview_pane = Paned::new(gtk4::Orientation::Horizontal);
        list_preview_pane.set_shrink_start_child(false);
        list_preview_pane.set_resize_end_child(false);
        list_preview_pane.set_start_child(Some(&file_scrolled));
        list_preview_pane.set_end_child(Some(&preview_scrolled));
        list_preview_pane.set_position(520);

        let search_bar = SearchBar::new();

        let sidebar_pane = Paned::new(gtk4::Orientation::Horizontal);
        sidebar_pane.set_shrink_start_child(false);
        sidebar_pane.set_resize_start_child(false);
        sidebar_pane.set_end_child(Some(&list_preview_pane));
        sidebar_pane.set_position(220);

        let breadcrumb_uri = Rc::new(RefCell::new(state.current_uri()));
        let breadcrumb_bar = build_breadcrumb_bar(Rc::clone(&breadcrumb_uri), Rc::new(|_| {}));

        let ui = Rc::new(UiContext {
            state: Rc::clone(&state),
            list_box: list_box.clone(),
            breadcrumb_bar: breadcrumb_bar.clone(),
            breadcrumb_uri,
            search_bar,
            preview,
        });

        let go_to_cell: Rc<RefCell<Option<Rc<dyn Fn(&str)>>>> = Rc::new(RefCell::new(None));
        let go_to = {
            let ui = Rc::clone(&ui);
            let go_to_cell = Rc::clone(&go_to_cell);
            Rc::new(move |uri: &str| {
                ui.state.set_current_uri(uri.to_string());
                if let Some(ref go_to) = *go_to_cell.borrow() {
                    ui.refresh(Rc::clone(go_to));
                }
            }) as Rc<dyn Fn(&str)>
        };
        *go_to_cell.borrow_mut() = Some(Rc::clone(&go_to));

        let sidebar = build_sidebar(Rc::clone(&go_to));
        sidebar_pane.set_start_child(Some(&sidebar));

        let right_panel = GtkBox::new(Orientation::Vertical, 0);
        right_panel.append(&breadcrumb_bar);
        right_panel.append(&ui.search_bar.widget);
        right_panel.append(&sidebar_pane);

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&right_panel);
        toast_overlay.set_child(Some(&content));

        ui.refresh(Rc::clone(&go_to));

        let run_search = {
            let ui = Rc::clone(&ui);
            let go_to = Rc::clone(&go_to);
            move || ui.refresh(Rc::clone(&go_to))
        };

        ui.search_bar.name_entry.connect_search_changed({
            let run_search = run_search.clone();
            move |_| run_search()
        });
        ui.search_bar.type_dropdown.connect_notify_local(Some("selected"), {
            let run_search = run_search.clone();
            move |_, _| run_search()
        });
        ui.search_bar.min_size_entry.connect_changed({
            let run_search = run_search.clone();
            move |_| run_search()
        });
        ui.search_bar.max_size_entry.connect_changed({
            let run_search = run_search.clone();
            move |_| run_search()
        });
        ui.search_bar.modified_days_entry.connect_changed({
            let run_search = run_search.clone();
            move |_| run_search()
        });

        list_box.connect_row_selected({
            let ui = Rc::clone(&ui);
            move |_, row| {
                let Some(row) = row else {
                    ui.preview.clear();
                    return;
                };
                let uri = row.widget_name();
                let resolved = if uri.starts_with("dir:") {
                    uri.strip_prefix("dir:").map(str::to_string)
                } else if uri.starts_with("file://") {
                    Some(uri.to_string())
                } else {
                    None
                };
                if let Some(uri) = resolved {
                    ui.preview.show_uri(&uri);
                } else {
                    ui.preview.clear();
                }
            }
        });

        up_button.connect_clicked({
            let state = Rc::clone(&state);
            let go_to = Rc::clone(&go_to);
            move |_| {
                if let Some(parent) = parent_uri(&state.current_uri()) {
                    go_to(&parent);
                }
            }
        });

        copy_button.connect_clicked({
            let state = Rc::clone(&state);
            let list_box = list_box.clone();
            let toast_overlay = toast_overlay.clone();
            move |_| {
                if let Some(uri) = selected_entry_uri(&list_box) {
                    *state.clipboard.borrow_mut() = Some(ClipboardEntry {
                        source_uri: uri,
                        operation: ClipboardOperation::Copy,
                    });
                    show_toast(&toast_overlay, "Copied to clipboard");
                } else {
                    show_toast(&toast_overlay, "Select a file or folder first");
                }
            }
        });

        cut_button.connect_clicked({
            let state = Rc::clone(&state);
            let list_box = list_box.clone();
            let toast_overlay = toast_overlay.clone();
            move |_| {
                if let Some(uri) = selected_entry_uri(&list_box) {
                    *state.clipboard.borrow_mut() = Some(ClipboardEntry {
                        source_uri: uri,
                        operation: ClipboardOperation::Move,
                    });
                    show_toast(&toast_overlay, "Ready to move");
                } else {
                    show_toast(&toast_overlay, "Select a file or folder first");
                }
            }
        });

        paste_button.connect_clicked({
            let state = Rc::clone(&state);
            let toast_overlay = toast_overlay.clone();
            let go_to = Rc::clone(&go_to);
            move |_| {
                let entry = state.clipboard.borrow().clone();
                let Some(entry) = entry else {
                    show_toast(&toast_overlay, "Nothing to paste");
                    return;
                };

                let dest_dir = state.current_uri();
                let result = match entry.operation {
                    ClipboardOperation::Copy => {
                        copy_into_directory(&entry.source_uri, &dest_dir)
                    }
                    ClipboardOperation::Move => {
                        move_into_directory(&entry.source_uri, &dest_dir)
                    }
                };

                match result {
                    Ok(()) => {
                        if entry.operation == ClipboardOperation::Move {
                            *state.clipboard.borrow_mut() = None;
                        }
                        show_toast(&toast_overlay, "Paste completed");
                        go_to(&dest_dir);
                    }
                    Err(err) => show_toast(&toast_overlay, &err),
                }
            }
        });

        delete_button.connect_clicked({
            let state = Rc::clone(&state);
            let list_box = list_box.clone();
            let toast_overlay = toast_overlay.clone();
            let window = window.clone();
            let go_to = Rc::clone(&go_to);
            move |_| {
                let Some(uri) = selected_entry_uri(&list_box) else {
                    show_toast(&toast_overlay, "Select a file or folder first");
                    return;
                };

                let dialog = adw::MessageDialog::builder()
                    .heading("Move to Trash?")
                    .body("This item will be moved to the trash.")
                    .transient_for(&window)
                    .modal(true)
                    .build();
                dialog.add_response("cancel", "Cancel");
                dialog.add_response("delete", "Move to Trash");
                dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                dialog.connect_response(None, {
                    let uri = uri.clone();
                    let state = Rc::clone(&state);
                    let toast_overlay = toast_overlay.clone();
                    let current_dir = state.current_uri();
                    let go_to = Rc::clone(&go_to);
                    move |dialog: &adw::MessageDialog, response: &str| {
                        if response == "delete" {
                            match trash_file(&uri) {
                                Ok(()) => {
                                    if state
                                        .clipboard
                                        .borrow()
                                        .as_ref()
                                        .is_some_and(|entry| entry.source_uri == uri)
                                    {
                                        *state.clipboard.borrow_mut() = None;
                                    }
                                    show_toast(&toast_overlay, "Moved to trash");
                                    go_to(&current_dir);
                                }
                                Err(err) => show_toast(&toast_overlay, &err),
                            }
                        }
                        dialog.destroy();
                    }
                });

                dialog.present();
            }
        });

        list_box.connect_row_activated({
            let go_to = Rc::clone(&go_to);
            move |_, row| {
                let uri = row.widget_name();
                let uri_str = uri.as_str();
                if uri_str.starts_with("dir:") {
                    let next_uri = uri_str.strip_prefix("dir:").unwrap_or(uri_str);
                    go_to(next_uri);
                } else if uri_str.starts_with("file://") && !is_directory(uri_str) {
                    open_file(uri_str);
                }
            }
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn selected_entry_uri(list_box: &gtk4::ListBox) -> Option<String> {
    let row = list_box.selected_row()?;
    let uri = row.widget_name();
    if uri.starts_with("dir:") {
        uri.strip_prefix("dir:").map(str::to_string)
    } else if uri.starts_with("file://") {
        Some(uri.to_string())
    } else {
        None
    }
}

fn open_file(uri: &str) {
    if let Err(err) = gio::AppInfo::launch_default_for_uri(
        uri,
        None::<&gio::AppLaunchContext>,
    ) {
        eprintln!("Failed to open {uri}: {err}");
    }
}

fn show_toast(toast_overlay: &adw::ToastOverlay, message: &str) {
    toast_overlay.add_toast(adw::Toast::new(message));
}

use gio;
