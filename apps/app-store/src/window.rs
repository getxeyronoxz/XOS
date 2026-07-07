use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::Path;
use std::collections::HashSet;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Button, ListBox};
use libadwaita as adw;
use adw::prelude::*;

const STATE_FILE: &str = "/etc/xos/installed_packs.state";
const FALLBACK_STATE: &str = "/tmp/xos_installed_packs.state";

struct ExtensionPack {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    icon: &'static str,
}

const PACKS: [ExtensionPack; 6] = [
    ExtensionPack {
        id: "developer",
        name: "Developer Toolkit",
        description: "Docker GUI, Git GUI, VS Code, Neovim",
        icon: "utilities-terminal-symbolic",
    },
    ExtensionPack {
        id: "security",
        name: "Security Toolkit",
        description: "Wireshark, nmap, Burp Community, KeePassXC",
        icon: "security-high-symbolic",
    },
    ExtensionPack {
        id: "research",
        name: "Research Toolkit",
        description: "Zotero, Obsidian, Xournal++, PDF annotator",
        icon: "accessories-dictionary-symbolic",
    },
    ExtensionPack {
        id: "ai",
        name: "AI Toolkit",
        description: "Ollama, LM Studio, local model manager",
        icon: "weather-clear-symbolic",
    },
    ExtensionPack {
        id: "media",
        name: "Media Toolkit",
        description: "OBS, Kdenlive, Audacity, screen recorder",
        icon: "audio-x-generic-symbolic",
    },
    ExtensionPack {
        id: "productivity",
        name: "Productivity Toolkit",
        description: "LibreOffice, Thunderbird, TaskWarrior",
        icon: "x-office-document-symbolic",
    },
];

pub struct AppStoreWindow {
    window: adw::ApplicationWindow,
    list_box: ListBox,
    installed: Rc<RefCell<HashSet<String>>>,
}

impl AppStoreWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("XOS Software Center")
            .default_width(680)
            .default_height(600)
            .build();

        let root_box = GtkBox::new(Orientation::Vertical, 0);
        window.set_content(Some(&root_box));

        // Header
        let header = adw::HeaderBar::new();
        header.set_title_widget(Some(&Label::new(Some("App Store"))));
        root_box.append(&header);

        // Scrolled view
        let scroll = gtk4::ScrolledWindow::new();
        scroll.set_vexpand(true);
        root_box.append(&scroll);

        let content_box = GtkBox::new(Orientation::Vertical, 16);
        content_box.set_margin_start(20);
        content_box.set_margin_end(20);
        content_box.set_margin_top(20);
        content_box.set_margin_bottom(20);
        scroll.set_child(Some(&content_box));

        // Title
        let title_lbl = Label::builder()
            .label("Extension Packs")
            .xalign(0.0)
            .build();
        title_lbl.add_css_class("title-1");
        content_box.append(&title_lbl);

        let desc_lbl = Label::builder()
            .label("Enhance XOS with pre-configured tools for development, security auditing, research, or content creation.")
            .xalign(0.0)
            .build();
        desc_lbl.add_css_class("body");
        content_box.append(&desc_lbl);

        // List
        let list_box = ListBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .css_classes(vec!["boxed-list".to_string()])
            .build();
        content_box.append(&list_box);

        // Read installed packs
        let installed = Rc::new(RefCell::new(read_installed_packs()));

        let mut win_self = Self {
            window,
            list_box,
            installed,
        };

        win_self.rebuild_list();
        win_self
    }

    pub fn present(&self) {
        self.window.present();
    }

    fn rebuild_list(&mut self) {
        // Clear list
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        for pack in &PACKS {
            let row = adw::ActionRow::builder()
                .title(pack.name)
                .subtitle(pack.description)
                .icon_name(pack.icon)
                .build();

            let is_installed = self.installed.borrow().contains(pack.id);
            
            let action_btn = Button::builder()
                .label(if is_installed { "Uninstall" } else { "Install" })
                .css_classes(vec![if is_installed { "destructive-action".to_string() } else { "suggested-action".to_string() }])
                .valign(gtk4::Align::Center)
                .build();

            row.add_suffix(&action_btn);

            action_btn.connect_clicked({
                let installed = Rc::clone(&self.installed);
                let pack_id = pack.id.to_string();
                let list_box = self.list_box.clone();
                let window = self.window.clone();
                move |btn| {
                    btn.set_sensitive(false);
                    let currently_installed = installed.borrow().contains(&pack_id);
                    if currently_installed {
                        installed.borrow_mut().remove(&pack_id);
                    } else {
                        installed.borrow_mut().insert(pack_id.clone());
                    }
                    save_installed_packs(&installed.borrow());
                    
                    // Rebuild list
                    let mut temp_self = AppStoreWindow {
                        window: window.clone(),
                        list_box: list_box.clone(),
                        installed: Rc::clone(&installed),
                    };
                    temp_self.rebuild_list();
                }
            });

            self.list_box.append(&row);
        }
    }
}

fn read_installed_packs() -> HashSet<String> {
    let path = if Path::new(STATE_FILE).exists() {
        STATE_FILE
    } else {
        FALLBACK_STATE
    };

    let mut set = HashSet::new();
    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let clean = line.trim();
            if !clean.is_empty() {
                set.insert(clean.to_string());
            }
        }
    }
    set
}

fn save_installed_packs(set: &HashSet<String>) {
    let path = if let Err(_) = fs::create_dir_all("/etc/xos") {
        FALLBACK_STATE
    } else {
        STATE_FILE
    };

    let mut content = String::new();
    for pack in set {
        content.push_str(pack);
        content.push_str("\n");
    }
    let _ = fs::write(path, content);
}
