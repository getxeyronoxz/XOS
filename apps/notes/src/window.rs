use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation, Paned, ScrolledWindow,
    SelectionMode, TextView,
};
use libadwaita as adw;
use adw::prelude::*;

use crate::storage::{create_note, delete_note, list_notes, read_note, write_note, NoteEntry};

struct EditorState {
    current_path: RefCell<Option<PathBuf>>,
}

impl EditorState {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            current_path: RefCell::new(None),
        })
    }
}

pub struct NotesWindow {
    window: adw::ApplicationWindow,
}

impl NotesWindow {
    pub fn new(app: &adw::Application) -> Self {
        let state = EditorState::new();

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Notes")
            .default_width(900)
            .default_height(640)
            .build();

        let toast_overlay = adw::ToastOverlay::new();
        window.set_content(Some(&toast_overlay));

        let header = adw::HeaderBar::new();
        let new_button = Button::from_icon_name("list-add-symbolic");
        new_button.set_tooltip_text(Some("New note"));
        let delete_button = Button::from_icon_name("user-trash-symbolic");
        delete_button.set_tooltip_text(Some("Delete note"));
        header.pack_start(&new_button);
        header.pack_end(&delete_button);

        let note_list = ListBox::new();
        note_list.set_selection_mode(SelectionMode::Single);
        note_list.add_css_class("navigation-sidebar");

        let list_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .min_content_width(220)
            .child(&note_list)
            .build();

        let editor = TextView::new();
        editor.set_monospace(true);
        editor.set_wrap_mode(gtk4::WrapMode::Word);
        editor.set_left_margin(12);
        editor.set_right_margin(12);
        editor.set_top_margin(12);
        editor.set_bottom_margin(12);

        let editor_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Automatic)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&editor)
            .build();

        let pane = Paned::new(gtk4::Orientation::Horizontal);
        pane.set_shrink_start_child(false);
        pane.set_resize_start_child(false);
        pane.set_start_child(Some(&list_scrolled));
        pane.set_end_child(Some(&editor_scrolled));
        pane.set_position(240);

        let content = GtkBox::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&pane);
        toast_overlay.set_child(Some(&content));

        let refresh_list = {
            let note_list = note_list.clone();
            let state = Rc::clone(&state);
            let editor = editor.clone();
            let toast_overlay = toast_overlay.clone();
            Rc::new(move |select_path: Option<&PathBuf>| {
                populate_note_list(&note_list, select_path, &state, &editor, &toast_overlay);
            })
        };

        refresh_list(None);

        new_button.connect_clicked({
            let refresh_list = Rc::clone(&refresh_list);
            let toast_overlay = toast_overlay.clone();
            move |_| match create_note() {
                Ok(note) => {
                    refresh_list(Some(&note.path));
                    toast_overlay.add_toast(adw::Toast::new("Created new note"));
                }
                Err(err) => toast_overlay.add_toast(adw::Toast::new(&err.to_string())),
            }
        });

        delete_button.connect_clicked({
            let state = Rc::clone(&state);
            let refresh_list = Rc::clone(&refresh_list);
            let toast_overlay = toast_overlay.clone();
            let editor = editor.clone();
            move |_| {
                let path = state.current_path.borrow().clone();
                let Some(path) = path else {
                    toast_overlay.add_toast(adw::Toast::new("No note selected"));
                    return;
                };
                match delete_note(&path) {
                    Ok(()) => {
                        editor.buffer().set_text("");
                        *state.current_path.borrow_mut() = None;
                        refresh_list(None);
                        toast_overlay.add_toast(adw::Toast::new("Note deleted"));
                    }
                    Err(err) => toast_overlay.add_toast(adw::Toast::new(&err.to_string())),
                }
            }
        });

        note_list.connect_row_selected({
            let state = Rc::clone(&state);
            let editor = editor.clone();
            let toast_overlay = toast_overlay.clone();
            move |_, row| {
                let Some(row) = row else { return };
                let path = PathBuf::from(row.widget_name());
                match read_note(&path) {
                    Ok(content) => {
                        editor.buffer().set_text(&content);
                        *state.current_path.borrow_mut() = Some(path);
                    }
                    Err(err) => toast_overlay.add_toast(adw::Toast::new(&err.to_string())),
                }
            }
        });

        editor.buffer().connect_changed({
            let state = Rc::clone(&state);
            let toast_overlay = toast_overlay.clone();
            move |buffer| {
                let path = state.current_path.borrow().clone();
                let Some(path) = path else { return };
                let (start, end) = buffer.bounds();
                let content = buffer.text(&start, &end, false);
                if let Err(err) = write_note(&path, &content.to_string()) {
                    toast_overlay.add_toast(adw::Toast::new(&err.to_string()));
                }
            }
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn populate_note_list(
    note_list: &ListBox,
    select_path: Option<&PathBuf>,
    state: &Rc<EditorState>,
    editor: &TextView,
    toast_overlay: &adw::ToastOverlay,
) {
    while let Some(row) = note_list.row_at_index(0) {
        note_list.remove(&row);
    }

    let notes = match list_notes() {
        Ok(notes) => notes,
        Err(err) => {
            toast_overlay.add_toast(adw::Toast::new(&err.to_string()));
            return;
        }
    };

    if notes.is_empty() {
        let row = ListBoxRow::new();
        row.set_selectable(false);
        row.set_activatable(false);
        row.set_child(Some(&Label::new(Some("No notes yet"))));
        note_list.append(&row);
        return;
    }

    let mut row_to_select: Option<ListBoxRow> = None;

    for note in notes {
        let row = ListBoxRow::new();
        row.set_widget_name(note.path.to_string_lossy().as_ref());

        let label = Label::new(Some(&note.title));
        label.set_xalign(0.0);
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        label.set_margin_bottom(8);
        row.set_child(Some(&label));
        note_list.append(&row);

        if select_path.is_some_and(|path| path == &note.path) {
            row_to_select = Some(row.clone());
        }
    }

    if let Some(row) = row_to_select {
        note_list.select_row(Some(&row));
    } else if let Some(first) = note_list.row_at_index(0) {
        note_list.select_row(Some(&first));
    } else {
        editor.buffer().set_text("");
        *state.current_path.borrow_mut() = None;
    }
}
