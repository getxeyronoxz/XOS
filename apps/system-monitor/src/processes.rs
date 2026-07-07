use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use libadwaita as adw;
use adw::prelude::*;

use crate::sysdata::ProcessInfo;

pub struct ProcessesPage {
    pub container: ScrolledWindow,
    list_box: ListBox,
}

impl ProcessesPage {
    pub fn new() -> Self {
        let main_box = GtkBox::new(Orientation::Vertical, 8);
        main_box.set_margin_start(24);
        main_box.set_margin_end(24);
        main_box.set_margin_top(24);
        main_box.set_margin_bottom(24);

        // Headers
        let header_hbox = GtkBox::new(Orientation::Horizontal, 12);
        header_hbox.set_margin_start(16);
        header_hbox.set_margin_end(16);
        header_hbox.set_margin_bottom(8);

        let name_hdr = Label::builder().label("Process Name").xalign(0.0).hexpand(true).build();
        name_hdr.add_css_class("heading");
        
        let pid_hdr = Label::builder().label("PID").xalign(0.0).width_chars(8).build();
        pid_hdr.add_css_class("heading");

        let cpu_hdr = Label::builder().label("CPU %").xalign(1.0).width_chars(10).build();
        cpu_hdr.add_css_class("heading");

        let mem_hdr = Label::builder().label("Memory").xalign(1.0).width_chars(12).build();
        mem_hdr.add_css_class("heading");

        header_hbox.append(&name_hdr);
        header_hbox.append(&pid_hdr);
        header_hbox.append(&cpu_hdr);
        header_hbox.append(&mem_hdr);
        
        main_box.append(&header_hbox);

        // List
        let list_box = ListBox::new();
        list_box.add_css_class("boxed-list");
        list_box.set_selection_mode(gtk4::SelectionMode::None);

        let list_scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&list_box)
            .vexpand(true)
            .build();

        main_box.append(&list_scrolled);

        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&main_box)
            .build();

        Self {
            container: scrolled,
            list_box,
        }
    }

    pub fn update(&mut self, processes: &[ProcessInfo]) {
        // Clear old rows
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        if processes.is_empty() {
            let row = ListBoxRow::new();
            let label = Label::new(Some("No processes found"));
            label.set_margin_top(12);
            label.set_margin_bottom(12);
            row.set_child(Some(&label));
            self.list_box.append(&row);
            return;
        }

        for proc in processes {
            let row = ListBoxRow::new();
            let hbox = GtkBox::new(Orientation::Horizontal, 12);
            hbox.set_margin_start(16);
            hbox.set_margin_end(16);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);

            let name_lbl = Label::builder()
                .label(&proc.name)
                .xalign(0.0)
                .hexpand(true)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();

            let pid_lbl = Label::builder()
                .label(&proc.pid)
                .xalign(0.0)
                .width_chars(8)
                .build();

            let cpu_lbl = Label::builder()
                .label(&format!("{:.1}%", proc.cpu))
                .xalign(1.0)
                .width_chars(10)
                .build();

            let mem_lbl = Label::builder()
                .label(&format_mem(proc.memory))
                .xalign(1.0)
                .width_chars(12)
                .build();

            hbox.append(&name_lbl);
            hbox.append(&pid_lbl);
            hbox.append(&cpu_lbl);
            hbox.append(&mem_lbl);

            row.set_child(Some(&hbox));
            self.list_box.append(&row);
        }
    }
}

fn format_mem(bytes: u64) -> String {
    let kb = bytes as f64 / 1024.0;
    if kb >= 1024.0 * 1024.0 {
        format!("{:.1} GB", kb / 1024.0 / 1024.0)
    } else if kb >= 1024.0 {
        format!("{:.1} MB", kb / 1024.0)
    } else {
        format!("{:.0} KB", kb)
    }
}
