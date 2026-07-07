use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, ComboBoxText, Entry, FileChooserAction, FileChooserDialog,
    Label, ListBox, ListBoxRow, Orientation, ResponseType, ScrolledWindow, Window,
};
use libadwaita as adw;
use adw::prelude::*;
use gtk4::gio;

use crate::archive::{compress_files, extract_archive, list_archive, ArchiveFormat};

pub struct ArchiveWindow {
    window: adw::ApplicationWindow,
}

struct CompressState {
    files_to_compress: RefCell<Vec<PathBuf>>,
}

impl ArchiveWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Archive Tool")
            .default_width(700)
            .default_height(550)
            .build();

        let toast_overlay = adw::ToastOverlay::new();
        window.set_content(Some(&toast_overlay));

        let toolbar_view = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&Label::new(Some("XOS Archive Tool"))));
        toolbar_view.add_top_bar(&header);

        let view_stack = adw::ViewStack::new();

        let view_switcher = adw::ViewSwitcher::new();
        view_switcher.set_stack(Some(&view_stack));
        header.set_title_widget(Some(&view_switcher));

        let compress_state = Rc::new(CompressState {
            files_to_compress: RefCell::new(Vec::new()),
        });

        let extract_box = build_extract_tab(&window, &toast_overlay);
        view_stack.add_titled(&extract_box, Some("extract"), "Extract");

        let compress_box = build_compress_tab(&window, &toast_overlay, compress_state);
        view_stack.add_titled(&compress_box, Some("compress"), "Compress");

        toolbar_view.set_content(Some(&view_stack));
        toast_overlay.set_child(Some(&toolbar_view));

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn open_file_dialog(parent: &impl IsA<gtk4::Window>, title: &str, action: FileChooserAction, select_multiple: bool) -> Option<Vec<PathBuf>> {
    let dialog = FileChooserDialog::builder()
        .title(title)
        .transient_for(parent)
        .action(action)
        .modal(true)
        .build();

    dialog.add_button("Cancel", ResponseType::Cancel);
    let accept_label = match action {
        FileChooserAction::Open => "Open",
        FileChooserAction::Save => "Save",
        FileChooserAction::SelectFolder => "Select",
        _ => "Ok",
    };
    dialog.add_button(accept_label, ResponseType::Accept);
    dialog.set_select_multiple(select_multiple);

    let loop_ctx = glib::MainLoop::new(None, false);
    let loop_ctx_clone = loop_ctx.clone();
    
    let res_cell = Rc::new(RefCell::new(None));
    let res_cell_clone = Rc::clone(&res_cell);
    
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Accept {
            let files = dialog.files();
            let mut paths = Vec::new();
            for i in 0..files.n_items() {
                if let Some(file) = files.item(i).and_then(|item| item.downcast::<gio::File>().ok()) {
                    if let Some(path) = file.path() {
                        paths.push(path);
                    }
                }
            }
            *res_cell_clone.borrow_mut() = Some(paths);
        }
        dialog.destroy();
        loop_ctx_clone.quit();
    });

    dialog.present();
    loop_ctx.run();
    
    let val = res_cell.borrow_mut().take();
    val
}

fn build_extract_tab(parent_window: &impl IsA<gtk4::Window>, toast_overlay: &adw::ToastOverlay) -> GtkBox {
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_start(18);
    main_box.set_margin_end(18);
    main_box.set_margin_top(18);
    main_box.set_margin_bottom(18);

    let select_row = GtkBox::new(Orientation::Horizontal, 8);
    let select_label = Label::new(Some("Select Archive:"));
    let path_label = Label::new(Some("No archive selected"));
    path_label.set_hexpand(true);
    path_label.set_xalign(0.0);
    path_label.add_css_class("dim-label");

    let selected_archive_path = Rc::new(RefCell::new(None::<PathBuf>));
    let selected_archive_path_clone = Rc::clone(&selected_archive_path);
    let parent_window_clone = parent_window.clone();
    
    let file_list_box = ListBox::new();
    file_list_box.set_selection_mode(gtk4::SelectionMode::None);

    let path_label_clone = path_label.clone();
    let file_list_box_clone = file_list_box.clone();

    let select_btn = Button::with_label("Browse...");
    select_btn.connect_clicked(move |_| {
        if let Some(paths) = open_file_dialog(&parent_window_clone, "Select Archive File", FileChooserAction::Open, false) {
            if let Some(path) = paths.into_iter().next() {
                path_label_clone.set_text(path.to_string_lossy().as_ref());
                *selected_archive_path_clone.borrow_mut() = Some(path.clone());
                
                while let Some(row) = file_list_box_clone.row_at_index(0) {
                    file_list_box_clone.remove(&row);
                }
                match list_archive(&path) {
                    Ok(entries) => {
                        if entries.is_empty() {
                            let row = ListBoxRow::new();
                            row.set_child(Some(&Label::new(Some("Archive is empty"))));
                            file_list_box_clone.append(&row);
                        } else {
                            for entry in entries.iter().take(200) {
                                let row = ListBoxRow::new();
                                let box_row = GtkBox::new(Orientation::Horizontal, 8);
                                let icon_name = if entry.is_dir { "folder-symbolic" } else { "text-x-generic-symbolic" };
                                let icon = gtk4::Image::from_icon_name(icon_name);
                                let name_label = Label::new(Some(&entry.name));
                                name_label.set_xalign(0.0);
                                let size_label = Label::new(Some(&format!("{} B", entry.size)));
                                size_label.add_css_class("dim-label");
                                
                                box_row.append(&icon);
                                box_row.append(&name_label);
                                box_row.append(&size_label);
                                box_row.set_margin_start(8);
                                box_row.set_margin_end(8);
                                box_row.set_margin_top(4);
                                box_row.set_margin_bottom(4);
                                row.set_child(Some(&box_row));
                                file_list_box_clone.append(&row);
                            }
                        }
                    }
                    Err(err) => {
                        let row = ListBoxRow::new();
                        row.set_child(Some(&Label::new(Some(&format!("Error reading archive: {err}")))));
                        file_list_box_clone.append(&row);
                    }
                }
            }
        }
    });

    select_row.append(&select_label);
    select_row.append(&path_label);
    select_row.append(&select_btn);

    let dest_row = GtkBox::new(Orientation::Horizontal, 8);
    let dest_label = Label::new(Some("Extract to:"));
    let dest_path_label = Label::new(None);
    dest_path_label.set_hexpand(true);
    dest_path_label.set_xalign(0.0);
    
    let default_dest = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/home"));
    dest_path_label.set_text(default_dest.to_string_lossy().as_ref());
    
    let selected_dest_path = Rc::new(RefCell::new(default_dest));
    let selected_dest_path_clone = Rc::clone(&selected_dest_path);
    let parent_window_clone2 = parent_window.clone();
    
    let dest_path_label_clone = dest_path_label.clone();
    let dest_btn = Button::with_label("Choose Folder...");
    dest_btn.connect_clicked(move |_| {
        if let Some(paths) = open_file_dialog(&parent_window_clone2, "Select Destination Folder", FileChooserAction::SelectFolder, false) {
            if let Some(path) = paths.into_iter().next() {
                dest_path_label_clone.set_text(path.to_string_lossy().as_ref());
                *selected_dest_path_clone.borrow_mut() = path;
            }
        }
    });

    dest_row.append(&dest_label);
    dest_row.append(&dest_path_label);
    dest_row.append(&dest_btn);

    let view_label = Label::new(Some("Archive Contents:"));
    view_label.set_xalign(0.0);
    view_label.add_css_class("title-4");

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&file_list_box)
        .hexpand(true)
        .vexpand(true)
        .build();

    let extract_btn = Button::builder()
        .label("Extract Archive")
        .css_classes(vec!["suggested-action".to_string()])
        .build();

    let toast_overlay_clone2 = toast_overlay.clone();
    let selected_archive_path_clone2 = Rc::clone(&selected_archive_path);
    let selected_dest_path_clone2 = Rc::clone(&selected_dest_path);

    extract_btn.connect_clicked(move |_| {
        let archive_borrow = selected_archive_path_clone2.borrow();
        let Some(archive_path) = archive_borrow.as_ref() else {
            toast_overlay_clone2.add_toast(adw::Toast::new("Please select an archive file first"));
            return;
        };
        let dest_path = selected_dest_path_clone2.borrow();
        
        toast_overlay_clone2.add_toast(adw::Toast::new("Extracting archive..."));
        
        match extract_archive(archive_path, &dest_path) {
            Ok(()) => {
                toast_overlay_clone2.add_toast(adw::Toast::new("Extraction completed successfully"));
            }
            Err(err) => {
                toast_overlay_clone2.add_toast(adw::Toast::new(&format!("Extraction failed: {err}")));
            }
        }
    });

    main_box.append(&select_row);
    main_box.append(&dest_row);
    main_box.append(&view_label);
    main_box.append(&scrolled);
    main_box.append(&extract_btn);

    main_box
}

fn build_compress_tab(parent_window: &impl IsA<gtk4::Window>, toast_overlay: &adw::ToastOverlay, state: Rc<CompressState>) -> GtkBox {
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_start(18);
    main_box.set_margin_end(18);
    main_box.set_margin_top(18);
    main_box.set_margin_bottom(18);

    let input_list_box = ListBox::new();
    input_list_box.set_selection_mode(gtk4::SelectionMode::None);

    let input_list_box_clone = input_list_box.clone();
    let state_clone = Rc::clone(&state);
    
    let refresh_input_list = move || {
        while let Some(row) = input_list_box_clone.row_at_index(0) {
            input_list_box_clone.remove(&row);
        }
        let files = state_clone.files_to_compress.borrow();
        if files.is_empty() {
            let row = ListBoxRow::new();
            row.set_child(Some(&Label::new(Some("No files selected for compression"))));
            input_list_box_clone.append(&row);
        } else {
            for (idx, path) in files.iter().enumerate() {
                let row = ListBoxRow::new();
                let row_box = GtkBox::new(Orientation::Horizontal, 8);
                let icon_name = if path.is_dir() { "folder-symbolic" } else { "text-x-generic-symbolic" };
                let icon = gtk4::Image::from_icon_name(icon_name);
                let name_label = Label::new(Some(path.to_string_lossy().as_ref()));
                name_label.set_xalign(0.0);
                name_label.set_hexpand(true);
                
                let remove_btn = Button::from_icon_name("user-trash-symbolic");
                remove_btn.set_tooltip_text(Some("Remove file"));
                
                let state_inner = Rc::clone(&state_clone);
                let remove_idx = idx;
                let row_clone = row.clone();
                let input_list_box_inner = input_list_box_clone.clone();
                
                remove_btn.connect_clicked(move |_| {
                    state_inner.files_to_compress.borrow_mut().remove(remove_idx);
                    input_list_box_inner.remove(&row_clone);
                });
                
                row_box.append(&icon);
                row_box.append(&name_label);
                row_box.append(&remove_btn);
                row_box.set_margin_start(8);
                row_box.set_margin_end(8);
                row_box.set_margin_top(4);
                row_box.set_margin_bottom(4);
                
                row.set_child(Some(&row_box));
                input_list_box_clone.append(&row);
            }
        }
    };

    let add_files_row = GtkBox::new(Orientation::Horizontal, 8);
    let select_files_btn = Button::with_label("Add Files...");
    let select_folders_btn = Button::with_label("Add Folders...");
    
    let parent_window_clone = parent_window.clone();
    let state_clone2 = Rc::clone(&state);
    let refresh_input_list_clone = Rc::new(refresh_input_list);
    let refresh_input_list_clone1 = Rc::clone(&refresh_input_list_clone);

    select_files_btn.connect_clicked(move |_| {
        if let Some(paths) = open_file_dialog(&parent_window_clone, "Add Files to Compress", FileChooserAction::Open, true) {
            for p in paths {
                if !state_clone2.files_to_compress.borrow().contains(&p) {
                    state_clone2.files_to_compress.borrow_mut().push(p);
                }
            }
            refresh_input_list_clone1();
        }
    });

    let parent_window_clone2 = parent_window.clone();
    let state_clone3 = Rc::clone(&state);
    let refresh_input_list_clone2 = Rc::clone(&refresh_input_list_clone);

    select_folders_btn.connect_clicked(move |_| {
        if let Some(paths) = open_file_dialog(&parent_window_clone2, "Add Folders to Compress", FileChooserAction::SelectFolder, true) {
            for p in paths {
                if !state_clone3.files_to_compress.borrow().contains(&p) {
                    state_clone3.files_to_compress.borrow_mut().push(p);
                }
            }
            refresh_input_list_clone2();
        }
    });

    add_files_row.append(&select_files_btn);
    add_files_row.append(&select_folders_btn);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .child(&input_list_box)
        .hexpand(true)
        .vexpand(true)
        .build();

    let name_row = GtkBox::new(Orientation::Horizontal, 8);
    let name_label = Label::new(Some("Archive Name:"));
    let name_entry = Entry::new();
    name_entry.set_hexpand(true);
    name_entry.set_text("archive");
    name_row.append(&name_label);
    name_row.append(&name_entry);

    let format_row = GtkBox::new(Orientation::Horizontal, 8);
    let format_label = Label::new(Some("Format:"));
    let format_dropdown = ComboBoxText::new();
    format_dropdown.append(Some("zip"), "ZIP (.zip)");
    format_dropdown.append(Some("tar.gz"), "TAR GZ (.tar.gz)");
    format_dropdown.set_active_id(Some("zip"));
    format_row.append(&format_label);
    format_row.append(&format_dropdown);

    let save_row = GtkBox::new(Orientation::Horizontal, 8);
    let save_label = Label::new(Some("Save in:"));
    let save_path_label = Label::new(None);
    save_path_label.set_hexpand(true);
    save_path_label.set_xalign(0.0);
    
    let default_dest = std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/home"));
    save_path_label.set_text(default_dest.to_string_lossy().as_ref());
    
    let selected_save_path = Rc::new(RefCell::new(default_dest));
    let selected_save_path_clone = Rc::clone(&selected_save_path);
    let parent_window_clone3 = parent_window.clone();
    let save_path_label_clone = save_path_label.clone();
    
    let save_btn = Button::with_label("Choose Folder...");
    save_btn.connect_clicked(move |_| {
        if let Some(paths) = open_file_dialog(&parent_window_clone3, "Select Output Directory", FileChooserAction::SelectFolder, false) {
            if let Some(path) = paths.into_iter().next() {
                save_path_label_clone.set_text(path.to_string_lossy().as_ref());
                *selected_save_path_clone.borrow_mut() = path;
            }
        }
    });

    save_row.append(&save_label);
    save_row.append(&save_path_label);
    save_row.append(&save_btn);

    let compress_btn = Button::builder()
        .label("Compress Files")
        .css_classes(vec!["suggested-action".to_string()])
        .build();

    let toast_overlay_clone = toast_overlay.clone();
    let state_clone4 = Rc::clone(&state);
    let name_entry_clone = name_entry.clone();
    let format_dropdown_clone = format_dropdown.clone();
    let selected_save_path_clone2 = Rc::clone(&selected_save_path);
    let refresh_input_list_clone3 = Rc::clone(&refresh_input_list_clone);

    compress_btn.connect_clicked(move |_| {
        let files = state_clone4.files_to_compress.borrow();
        if files.is_empty() {
            toast_overlay_clone.add_toast(adw::Toast::new("No files selected to compress"));
            return;
        }

        let name = name_entry_clone.text().to_string();
        if name.is_empty() {
            toast_overlay_clone.add_toast(adw::Toast::new("Please specify an archive name"));
            return;
        }

        let format_id = format_dropdown_clone.active_id().unwrap_or_else(|| glib::GString::from("zip"));
        let ext = if format_id == "tar.gz" { ".tar.gz" } else { ".zip" };
        let format = if format_id == "tar.gz" { ArchiveFormat::TarGz } else { ArchiveFormat::Zip };
        
        let output_file = selected_save_path_clone2.borrow().join(format!("{name}{ext}"));
        
        toast_overlay_clone.add_toast(adw::Toast::new("Compressing files..."));

        match compress_files(&files, &output_file, format) {
            Ok(()) => {
                toast_overlay_clone.add_toast(adw::Toast::new(&format!("Archive created successfully: {}", output_file.display())));
                state_clone4.files_to_compress.borrow_mut().clear();
                refresh_input_list_clone3();
            }
            Err(err) => {
                toast_overlay_clone.add_toast(adw::Toast::new(&format!("Compression failed: {err}")));
            }
        }
    });

    main_box.append(&add_files_row);
    main_box.append(&scrolled);
    main_box.append(&name_row);
    main_box.append(&format_row);
    main_box.append(&save_row);
    main_box.append(&compress_btn);

    refresh_input_list_clone();

    main_box
}
